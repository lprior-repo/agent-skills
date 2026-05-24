# Fjall Troubleshooting Reference

Diagnostic procedures for Fjall-backed orchestrators. Each section follows
symptom -> diagnosis -> fix format with immediate remediation and long-term
prevention strategies.

## Table of Contents

1. [Cold Start Issues](#1-cold-start-issues)
2. [Write Stalls](#2-write-stalls)
3. [High Read Latency](#3-high-read-latency)
4. [Memory Issues](#4-memory-issues)
5. [KV Separation Problems](#5-kv-separation-problems)
6. [Compaction Issues](#6-compaction-issues)
7. [WAL and Durability Issues](#7-wal-and-durability-issues)
8. [Prefix Scan Performance](#8-prefix-scan-performance)
9. [Database Locking Errors](#9-database-locking-errors)
10. [Recovery Procedures](#10-recovery-procedures)

---

## 1. Cold Start Issues

### Observable Symptoms

- Latency spike of 5-30+ seconds after process restart before the database
  becomes responsive.
- RSS memory spike to ~2 GB for a 1.3 GB dataset that uses KV separation,
  visible via `top` or process metrics.
- Blob index loading dominates startup CPU profile.
- First reads after restart are orders of magnitude slower than steady-state
  reads.
- Logs show long gap between "opening database" and "database ready" messages.

### Root Cause Analysis

1. **Check KV separation blob index size.** KV-separated keyspaces store a
   blob index in the LSM-tree. On startup, Fjall must load and materialize
   this index into memory. For a 1.3 GB dataset with many blob references,
   this can consume 2 GB of RSS during load before settling.

2. **Check cache population.** The unified block cache starts empty. Every
   block miss triggers disk I/O. Compaction metadata, Bloom filter blocks,
   and index blocks all compete for cache space during the ramp-up period.

3. **Check the number of SSTable files.** A large number of small SSTables
   (e.g., from frequent small flushes) increases the number of index blocks
   that must be loaded during startup.

4. **Profile with `perf record`** during startup to confirm time is spent in
   blob index loading versus general SSTable open.

Diagnostic commands:
```bash
# Check database directory size and file count
find /path/to/db -name "*.sst" | wc -l
du -sh /path/to/db/blobs/

# Monitor RSS during startup
/usr/bin/time -v target/release/veloxide 2>&1 | grep "Maximum resident"
```

### Immediate Fix

- **Increase `cache_size`.** The unified block cache should be large enough to
  hold the blob index plus hot SSTable blocks. Set to at least 25% of system
  RAM, or higher if the dataset is mostly KV-separated:
  ```rust
  let db = Database::builder(path)
      .cache_size(512 * 1024 * 1024) // 512 MiB minimum for blob-heavy workloads
      .open()?;
  ```

- **Warm the cache on boot.** Perform a sequential scan of hot keyspaces
  immediately after opening the database. This forces block loads while the
  application is not yet serving traffic:
  ```rust
  // During startup, before accepting traffic:
  let db = Database::builder(path).open()?;
  let events = db.keyspace("events", KeyspaceCreateOptions::default())?;
  for guard in events.prefix(&instance_prefix) {
      let _ = guard.key()?;  // Touch keys to populate block cache
  }
  ```

- **Consider a smaller `separation_threshold`.** If many small values are
  being separated unnecessarily, lower the threshold so only truly large
  values go to blob storage, reducing blob index size:
  ```rust
  let opts = KeyspaceCreateOptions::default()
      .with_kv_separation(
          KvSeparationOptions::default()
              .separation_threshold(4096) // Only separate values >= 4 KiB
      );
  ```

### Long-Term Prevention

- Monitor cache hit rate as a health metric. If hit rate drops below 90%
  during steady state, the cache is undersized.
- Pre-compute cache size as `max(blob_index_size * 1.5, dataset_size * 0.20)`.
- Schedule restarts during low-traffic windows and perform cache warming
  before marking the instance healthy.
- For very large datasets, consider sharding across multiple database
  instances to reduce per-instance cold start cost.

### Relevant Fjall Version Notes

- V2.6 introduced KV separation. V3.0 refined blob GC and index handling.
- V3.1 added configurable `separation_threshold` (previously fixed).
- The blob index is not persisted across restarts in V2.x; V3.0 added partial
  persistence but full reload is still required.

---

## 2. Write Stalls

### Observable Symptoms

- Write latency gradually increases from single-digit milliseconds to hundreds
  of milliseconds or seconds.
- Eventually writes block entirely. Application-level timeouts fire. Queue
  depths in `BudgetQueues` grow unboundedly until backpressure activates.
- Metrics show `L0 SSTable count` climbing continuously without reduction.
- Compaction thread CPU usage is at or near 100%.
- Disk I/O bandwidth is saturated (check `iostat -x 1`).

### Root Cause Analysis

1. **L0 accumulation.** L0 SSTables accumulate faster than compaction can
   merge them into L1. When L0 count exceeds the stall threshold, writes are
   deliberately slowed to prevent unbounded growth.

2. **Check L0 count and compaction trigger:**
   ```rust
   // Inspect via the database metrics
   // L0 segment count is visible in internal stats
   ```

3. **Check disk I/O bandwidth.** Compaction reads SSTables, merges them, and
   writes new SSTables. If the disk cannot sustain the required read+write
   bandwidth, compaction falls behind:
   ```bash
   iostat -x 1 10
   # Look for %util near 100% or high await times
   ```

4. **Check memtable size.** If `max_memtable_size` is too small, memtables
   flush to L0 too frequently, creating many small SSTables that increase
   compaction overhead.

5. **Check for ephemeral data.** Time-series or TTL data written to a Leveled
   compaction keyspace creates unnecessary compaction work. FIFO compaction
   is more appropriate.

### Immediate Fix

- **Increase `max_memtable_size`.** Larger memtables mean fewer flushes, fewer
  L0 SSTables, and less compaction pressure:
  ```rust
  let db = Database::builder(path)
      .max_memtable_size(64 * 1024 * 1024) // 64 MiB per memtable
      .open()?;
  ```

- **Tune `l0_compaction_trigger`.** Start compaction earlier so L0 does not
  accumulate as many segments before compaction kicks in:
  ```rust
  // Trigger compaction when L0 reaches 4 segments (default is higher)
  // This reduces the peak L0 depth at the cost of more frequent compaction
  ```

- **Check and address disk I/O bottlenecks.** If on HDD or shared cloud disk,
  consider moving to NVMe or provisioning more IOPS. Compaction is
  I/O-intensive by nature.

- **Use FIFO compaction for ephemeral data.** Data that has a natural TTL or
  is transient should not go through Leveled compaction:
  ```rust
  let opts = KeyspaceCreateOptions::default()
      .with_compaction_strategy(CompactionStrategy::Fifo(FifoCompactionOptions::default()));
  ```

### Long-Term Prevention

- Monitor L0 segment count. Alert when it exceeds 50% of the stall threshold.
- Size memtables relative to write throughput: `memtable_size >= write_rate *
  acceptable_flush_interval`.
- Provision disk I/O headroom of 2x steady-state write rate to absorb
  compaction bursts.
- Use `BudgetQueues` with appropriate backpressure thresholds to shed load
  before stalls propagate to the application.

### Relevant Fjall Version Notes

- V2.0 introduced configurable compaction strategies.
- V3.0 improved compaction scheduling to reduce stall frequency.
- FIFO compaction was stabilized in V2.6.

---

## 3. High Read Latency

### Observable Symptoms

- Point reads that previously took microseconds now take milliseconds or tens
  of milliseconds.
- Prefix scans show highly variable latency. Some reads are fast (cached),
  others are slow (disk seek).
- P99 read latency is 10-100x higher than P50.
- Bloom filter false-positive rate appears elevated (more disk I/O per read).

### Root Cause Analysis

1. **Check L0 segment count.** L0 segments can have overlapping key ranges.
  Each point read must check every L0 segment. If there are 50 L0 segments,
  a single point read does 50 Bloom filter checks and potentially 50 disk
  seeks:
  ```bash
  # Look at SSTable distribution across levels
  find /path/to/db -name "*.sst" -path "*/L0*" | wc -l
  ```

2. **Check Bloom filter configuration.** If Bloom filter bits-per-key is too
  low, false positives increase, causing unnecessary block reads:
  ```rust
  // Default is ~10 bits/key. For read-heavy workloads, increase to 20.
  ```

3. **Check cache miss rate.** If the unified cache is too small, index blocks
  and Bloom filter blocks are evicted, forcing repeated disk reads:
  ```bash
  # Monitor cache utilization if exposed via metrics
  ```

4. **Check for long-held snapshots.** MVCC snapshots prevent compaction from
  reclaiming space. SSTables that should be compacted away are kept alive,
  increasing read amplification:
  ```rust
  // Check if any snapshots are held longer than expected
  // Snapshots hold a seqno, preventing GC of anything after that seqno
  ```

5. **Profile the read path.** Use `perf` or `tracing` spans to identify
  whether latency is in Bloom filter checks, block reads, or decompression.

### Immediate Fix

- **Fix compaction lag** (see Section 6). High L0 count is the most common
  cause of read latency spikes.

- **Increase Bloom filter bits-per-key.** For read-heavy keyspaces, set
  bits-per-key to 15-20:
  ```rust
  let opts = KeyspaceCreateOptions::default()
      .with_bloom_filter_bits_per_key(20);
  ```

- **Increase cache size.** Ensure the cache can hold at least all index blocks
  and Bloom filter blocks for hot keyspaces.

- **Release long-lived snapshots.** Find and drop any snapshots that have been
  held for longer than necessary. Snapshots used for batch processing should
  be scoped to the batch, not held across batches.

### Long-Term Prevention

- Monitor L0 count and compaction lag as first-class metrics.
- For point-read-heavy keyspaces, use `expect_point_read_hits(true)` on Lmax
  to disable Bloom filters on the deepest level (saves ~1.25 GB per billion
  keys with no false positives since all keys are present).
- Right-size the cache for the working set, not the total dataset.
- Keep snapshot lifetimes bounded. Never hold a snapshot across an `await`
  point in async code.

### Relevant Fjall Version Notes

- V3.0 added `expect_point_read_hits()` for Bloom filter optimization.
- V2.8 improved Bloom filter accuracy for small keyspaces.
- V3.1 introduced compaction filters which can reduce tombstone-related read
  amplification.

---

## 4. Memory Issues

### Observable Symptoms

- RSS grows continuously and never stabilizes, even during idle periods.
- OOM kills on constrained environments (containers, small VMs).
- `Slice` references appear in heap profiles holding large blocks alive.
- Cache eviction is not reducing memory as expected.

### Root Cause Analysis

1. **Profile with `jemalloc` or `tikv-jemalloc-heap`:**
  ```rust
  // Add to Cargo.toml: tikv-jemallocator = "0.6"
  // Then dump heap profiles to identify what is holding memory
  ```

2. **Check for `Slice` references holding blocks alive.** Fjall `Slice` is a
  zero-copy reference to a block in the cache. If application code holds
  `Slice` values in long-lived data structures (HashMaps, static variables),
  the underlying cache blocks cannot be evicted, effectively bypassing the
  cache size limit:
  ```rust
  // PROBLEM: Slice held in a long-lived map
  let map: HashMap<Vec<u8>, Slice> = HashMap::new();
  // Even after the cache wants to evict, these blocks stay pinned
  ```

3. **Check for long-lived snapshots.** Snapshots prevent block eviction for
  any SSTable referenced by the snapshot's seqno. A snapshot held for hours
  prevents compaction output from being cleaned up, causing unbounded growth.

4. **Check cache size configuration.** If `cache_size` is unset, it defaults
  to a value that may be too large for the environment.

5. **Check for large value accumulation.** If values grow over time (e.g.,
  event payloads), the memtable and SSTable block sizes increase, consuming
  more memory.

### Immediate Fix

- **Cap `cache_size` explicitly.** Set it to a known bound based on available
  memory:
  ```rust
  let db = Database::builder(path)
      .cache_size(256 * 1024 * 1024) // Hard cap at 256 MiB
      .open()?;
  ```

- **Copy long-lived values to owned memory.** Any `Slice` that will outlive
  the immediate read operation must be copied:
  ```rust
  // CORRECT: Copy to Vec<u8> for long-term storage
  let val: Vec<u8> = slice.to_vec();
  // Now the cache block can be evicted freely
  ```

- **Keep snapshots short-lived.** Acquire a snapshot, perform the operation,
  drop the snapshot. Never store snapshots in `Arc`, static variables, or
  long-lived collections.

### Long-Term Prevention

- Use `tikv-jemalloc` for production builds. Its heap profiling capabilities
  make memory leak diagnosis straightforward.
- Audit all `Slice` usage: any `Slice` that escapes the function where it was
  created should be converted to `Vec<u8>`.
- Set `cache_size` explicitly in all environments, especially containers
  where memory limits are enforced via cgroups.
- Monitor RSS and alert when it exceeds 2x the configured cache size.

### Relevant Fjall Version Notes

- V3.0 introduced the `Slice` type as a zero-copy view. V2.x used `Vec<u8>`
  for all reads. The zero-copy design improves throughput but introduces
  block-pinning risk.
- V3.0 unified block cache replaced per-keyspace caches, improving efficiency
  but requiring careful sizing.

---

## 5. KV Separation Problems

### Observable Symptoms

- Cold start latency spike specific to keyspaces with KV separation enabled
  (see Section 1).
- Data corruption or missing values after calling `clear()` on a KV-separated
  keyspace.
- Blob files (.blob) growing without bound. Disk usage increases even though
  data is being deleted or overwritten.
- Read errors returning stale or missing values for recently written keys.

### Root Cause Analysis

1. **Cold start spike.** The blob index must be loaded into memory on startup.
  For large datasets, this dominates startup time and memory. See Section 1
  for full diagnosis.

2. **`clear()` corruption (Bug #277).** Calling `clear()` on a KV-separated
  keyspace corrupts the blob index. The LSM-tree entries are removed but the
  blob files are not properly cleaned up. Subsequent reads may return wrong
  values or errors:
  ```rust
  // NEVER DO THIS on a KV-separated keyspace:
  kv_separated_ks.clear().unwrap();
  // This corrupts the blob index (fjall issue #277)
  ```

3. **Blob GC lag.** Garbage collection of blob files runs during compaction.
  If compaction is slow or if the `staleness_threshold` is set too high, dead
  blob references accumulate, and blob files grow without bound.

4. **Check blob directory size vs. LSM-tree size:**
  ```bash
  du -sh /path/to/db/blobs/
  du -sh /path/to/db/
  # If blobs/ is much larger than expected, GC is lagging
  ```

### Immediate Fix

- **Cap the cache.** Ensure the cache can hold the blob index with room for
  hot blocks. See Section 1 for sizing guidance.

- **Never call `clear()` on KV-separated keyspaces.** This is a known bug
  (#277). To delete all data in a KV-separated keyspace, either:
  - Drop and recreate the keyspace entirely.
  - Delete the database directory and reinitialize.
  - Iterate and remove individual keys (slower but safe).

- **Monitor `staleness_threshold`.** The blob GC only reclaims blob references
  that have been stale for longer than the threshold. If the threshold is too
  high, blob files accumulate. Lower it for workloads with high churn:
  ```rust
  let opts = KvSeparationOptions::default()
      .staleness_threshold(Duration::from_secs(300)); // 5 minutes
  ```

### Long-Term Prevention

- Track blob directory size as a monitoring metric. Alert when it exceeds
  2x the logical data size.
- Run compaction frequently enough to trigger blob GC. If compaction is
  throttled, blob reclamation falls behind.
- Before using KV separation, verify that values exceed the
  `separation_threshold`. Separating small values wastes I/O and inflates
  the blob index.
- Document the `clear()` prohibition prominently in code comments near any
  KV-separated keyspace.

### Relevant Fjall Version Notes

- V2.6 introduced KV separation.
- Bug #277 (`clear()` corruption) is present in V2.6 through V3.0. Check
  changelog for fix status.
- V3.0 added configurable `staleness_threshold` for blob GC.
- V3.1 improved blob GC scheduling during compaction.

---

## 6. Compaction Issues

### Observable Symptoms

- L0 SSTable count grows continuously, never decreasing.
- Tombstone accumulation: deleted keys still appear in scans or cause
  unexpected behavior.
- Write amplification is high. Disk writes are 10-50x the logical write rate.
- Compaction thread(s) consume high CPU continuously without reducing L0
  depth.

### Root Cause Analysis

1. **Monitor L0 count and compaction throughput:**
  ```bash
  # Count L0 SSTables
  find /path/to/db -name "*.sst" -path "*L0*" | wc -l

  # Watch compaction I/O
  iostat -x 5
  ```

2. **Check level fanout.** A high `level_fanout` (e.g., 10) means each level
  is 10x the size of the previous level. Compaction from Ln to Ln+1 involves
  reading and rewriting 10x the data, amplifying writes.

3. **Check tombstone accumulation.** In Leveled compaction, tombstones are
  only dropped during compaction when no snapshot references the deleted key's
  seqno. If snapshots are long-lived, tombstones accumulate in all levels,
  increasing read amplification and space amplification.

4. **Check for small, frequent flushes.** Memtables that are too small cause
  frequent flushes, creating many small L0 SSTables. Each one must be
  compacted, increasing overhead.

5. **Check compaction thread count.** Fjall defaults to a limited number of
  compaction threads. On machines with many cores and heavy write loads, this
  may be insufficient.

### Immediate Fix

- **Tune `level_fanout`.** A fanout of 4 reduces write amplification at the
  cost of more levels:
  ```rust
  let db = Database::builder(path)
      .level_fanout(4) // Default is 10; lower reduces write amplification
      .open()?;
  ```

- **Use compaction filters for TTL/GDPR.** V3.1 supports compaction filters
  that can drop expired entries during compaction, preventing tombstone
  accumulation:
  ```rust
  let opts = KeyspaceCreateOptions::default()
      .with_compaction_filter(|key, value| {
          // Return false to drop the entry during compaction
          if is_expired(value) { false } else { true }
      });
  ```

- **Increase memtable size.** Larger memtables produce fewer, larger SSTables
  that are more efficient to compact.

- **Use FIFO compaction for TTL data.** If data naturally expires, FIFO
  compaction drops entire SSTables after a size or time limit, avoiding
  tombstone overhead entirely.

### Long-Term Prevention

- Monitor write amplification ratio (bytes written to disk / bytes written by
  application). Alert when it exceeds 30x.
- Right-size memtables for write rate: larger memtables amortize compaction
  cost.
- Use FIFO compaction for any keyspace where data is transient or has a
  known TTL.
- Keep snapshots short-lived to allow tombstone collection.

### Relevant Fjall Version Notes

- V2.0 introduced configurable compaction strategies and level fanout.
- V3.1 added compaction filters (callback-based entry retention during
  compaction).
- V3.0 improved compaction scheduling to reduce unnecessary re-compaction.

---

## 7. WAL and Durability Issues

### Observable Symptoms

- Data loss after power failure or hard crash. Events that were acknowledged
  as written are missing after restart.
- Write throughput is significantly lower than expected. Every write triggers
  an fsync.
- WAL (journal) files grow without bound.

### Root Cause Analysis

1. **Verify `PersistMode` usage.** Fjall supports multiple durability levels:
  - `PersistMode::SyncAll` -- fsync after every write. Maximum durability,
    minimum throughput.
  - `PersistMode::Buffer` -- batch writes in the OS page cache. Higher
    throughput, potential data loss on power failure.
  - `PersistMode::None` -- no durability guarantee. In-memory only.

2. **Check if individual inserts use `SyncAll`.** If every `ks.insert()` call
  triggers an fsync, throughput collapses:
  ```rust
  // PROBLEM: Each insert fsyncs independently
  ks.insert("key1", "val1")?; // fsync
  ks.insert("key2", "val2")?; // fsync
  ks.insert("key3", "val3")?; // fsync
  ```

3. **Check WAL file count.** WAL files should be cleaned up after successful
  flush to SSTables. If they accumulate, flush is not completing properly:
  ```bash
  find /path/to/db -name "journal*" | wc -l
  ```

4. **Check for unflushed memtables on crash.** If the process crashes before
  a memtable is flushed to an SSTable, the WAL is replayed on recovery. If the
  WAL is corrupt (partial write during crash), recovery may fail.

### Immediate Fix

- **Use `OwnedWriteBatch` to amortize fsync.** Batch multiple writes into a
  single atomic commit with one fsync:
  ```rust
  let mut batch = db.batch();
  batch.insert(events, "key1", "val1");
  batch.insert(events, "key2", "val2");
  batch.insert(index, "key3", "val3");
  batch.commit()?; // Single fsync for all three writes
  ```

- **Use `PersistMode::SyncAll` only for critical writes.** Event appends
  (workflow state transitions) need full durability. Bulk blob ingestion or
  projection updates can use `Buffer`:
  ```rust
  // Critical: event append
  let mut batch = db.batch();
  batch.insert(events, event_key, event_bytes);
  batch.commit()?; // Defaults to SyncAll for the events keyspace

  // Non-critical: projection update
  let mut batch = db.batch();
  batch.insert(projections, proj_key, proj_bytes);
  // Consider Buffer mode for projections if some loss is acceptable
  ```

- **Verify WAL replay on startup.** After a crash, Fjall replays the WAL
  before opening. If this hangs or fails, the WAL may be corrupt. See
  Section 10 for recovery procedures.

### Long-Term Prevention

- Always use `OwnedWriteBatch` for multi-key or multi-keyspace writes. Never
  issue individual `insert()` calls for related data.
- Size batches to balance latency and throughput: 100-1000 keys per batch is
  typical.
- Test crash recovery regularly: write data, kill the process with `kill -9`,
  restart, verify data integrity.
- Monitor WAL file count. Accumulation indicates flush problems.

### Relevant Fjall Version Notes

- V2.0 introduced `WriteBatch` (renamed to batch API in V3).
- V3.0 reorganized the batch API as `db.batch()` returning a builder.
- WAL format changed between V2.x and V3.0. Databases created with V2.x must
  be migrated.

---

## 8. Prefix Scan Performance

### Observable Symptoms

- Prefix scans are slow even for small key ranges (hundreds of keys).
- Full table scans are faster than targeted prefix scans (indicating the
  prefix optimization is not working).
- Missing data in scans: keys that exist via point reads do not appear in
  prefix scans.
- Scan performance varies dramatically depending on key distribution.

### Root Cause Analysis

1. **Check key encoding.** Fjall uses lexicographic byte comparison. If keys
  use little-endian integer encoding, numerical order does not match byte
  order, breaking prefix locality:
  ```rust
  // WRONG: Little-endian breaks locality
  let key = [prefix, &instance_id.to_le_bytes()].concat();

  // CORRECT: Big-endian preserves numerical order
  let key = [prefix, &instance_id.to_be_bytes()].concat();
  ```

2. **Check prefix delimiter.** If variable-length components are concatenated
  without a delimiter, prefix boundaries are ambiguous:
  ```rust
  // WRONG: "abc" + "def" == "abcdef" == "abcd" + "ef"
  let key = format!("{}{}", prefix, suffix);

  // CORRECT: Null byte delimiter
  let key = [prefix.as_bytes(), &[0u8], suffix.as_bytes()].concat();
  ```

3. **Verify monotonic key generation.** Fjall optimizes prefix scans when
  keys within a prefix are generated monotonically. If keys are written in
  random order within a prefix, the O(1) seek optimization degrades to O(n).

4. **Check for overlapping prefix ranges across SSTables.** If compaction has
  not merged all keys for a prefix into the same SSTable, the scan must
  check multiple files.

### Immediate Fix

- **Use big-endian encoding for all integer key components:**
  ```rust
  // Veloxide key format (all big-endian):
  // events:     [InstanceId_16B | SequenceNumber_8B]       = 24B
  // instances:  [StatusByte_1B | CreatedAt_8B | Id_16B]    = 25B
  // timers:     [FireAtMs_8B | InstanceId_16B | TimerId_16B] = 40B
  let key = [id.as_bytes(), &seq.to_be_bytes()].concat();
  ```

- **Use null-delimited prefixes for variable-length components:**
  ```rust
  let prefix = [tenant_id.as_bytes(), &[0u8]].concat();
  for guard in ks.prefix(&prefix) {
      // Only keys matching tenant_id\x00 prefix
  }
  ```

- **Ensure monotonic key generation within a prefix.** Sequence numbers
  within an instance should always increase. If out-of-order writes are
  necessary, accept that prefix scans will be slower.

### Long-Term Prevention

- Establish a key design convention document for the project. Veloxide uses
  fixed-size big-endian keys (see SKILL.md).
- Add lint checks for `to_le_bytes()` in key construction code.
- Test prefix scan performance with representative data distributions.
- For new keyspaces, prototype the key format and verify prefix scan
  efficiency before committing to the schema.

### Relevant Fjall Version Notes

- V2.0 added prefix scan support with Bloom filter optimization.
- V3.0 introduced the `Guard` iterator API that defers blob loading.
- V3.0 improved prefix truncation in SSTable index blocks for better prefix
  scan locality.

---

## 9. Database Locking Errors

### Observable Symptoms

- Error on `Database::open()`: file lock error, "database is locked", or
  "unable to acquire lock".
- Second process cannot open the same database path.
- Application fails to start after a crash, reporting lock errors even though
  no other process is running.

### Root Cause Analysis

1. **Check for competing processes.** Fjall V3 uses an exclusive file lock.
  Only one process can open a database at a time:
  ```bash
  # Find processes with the database open
  lsof +D /path/to/db 2>/dev/null
  fuser -v /path/to/db/*.lock 2>/dev/null
  ```

2. **Check for stale lock files after a crash.** On ungraceful termination
  (kill -9, kernel panic), the lock file may not be cleaned up. Fjall
  normally handles this via lock file metadata, but in rare cases (NFS,
  certain filesystems) the stale lock may persist.

3. **Check for zombie processes.** A parent process that forked but did not
  exec may inherit the file descriptor:
  ```bash
  ps aux | grep veloxide
  ```

4. **Check filesystem type.** Some network filesystems (NFS, CIFS) do not
  support POSIX file locks reliably.

### Immediate Fix

- **Ensure single process per database.** This is a hard constraint in V3.
  Use process supervision (systemd, supervisord) to guarantee at most one
  instance.

- **Kill competing processes:**
  ```bash
  # Identify and kill any process holding the lock
  fuser -k /path/to/db
  ```

- **Remove stale lock file (last resort, only when no other process exists):**
  ```bash
  # ONLY if no other process is running:
  rm /path/to/db/LOCK
  # Or the equivalent lock file for your Fjall version
  ```

### Long-Term Prevention

- Use a process supervisor that prevents duplicate instances (systemd with
  `Type=notify`, or a PID file with advisory locking).
- Never share a database path between multiple processes or containers.
  Each veloxide instance must have its own data directory.
- For development, use separate database paths per test to avoid lock
  contention in parallel test runs.

### Relevant Fjall Version Notes

- V3.0 introduced exclusive file locking. V2.x allowed multiple readers but
  this was removed for consistency guarantees.
- Lock file format changed between V2 and V3. Mixing versions on the same
  database directory causes errors.

---

## 10. Recovery Procedures

### Dirty Shutdown Recovery

**Scenario:** Process killed with `kill -9`, OOM, kernel panic, or power loss.

**Recovery process:**

1. **Fjall automatically replays the WAL on next open.** No manual
   intervention is needed for clean recovery:
   ```rust
   // Just open the database normally. WAL replay is automatic.
   let db = Database::builder(path).open()?;
   // If this succeeds, recovery was successful.
   ```

2. **If WAL replay fails** (corrupt journal), Fjall will return an error on
   open. At this point:
   ```bash
   # Check the journal files for corruption
   ls -la /path/to/db/journal/

   # Attempt to read journal entries (if tooling is available)
   # If journal is truncated, Fjall may be able to recover partial entries
   ```

3. **If the database will not open at all:**
   ```bash
   # Back up the entire database directory before attempting recovery
   cp -r /path/to/db /path/to/db.corrupt-backup

   # Try removing the corrupt journal file
   rm /path/to/db/journal/*.log

   # Attempt to open again
   # WARNING: You will lose any data that was only in the WAL
   ```

### WAL Replay Details

- On open, Fjall reads the WAL sequentially and reapplies all operations to
  memtables.
- The WAL is idempotent: replaying it multiple times produces the same result.
- WAL replay time is proportional to WAL size. A large WAL (many unflushed
  writes) means longer recovery time.
- After successful replay, memtables are flushed to SSTables and the WAL is
  truncated.

### Checkpoint Restoration

**Scenario:** Database corruption requires restoration from a known-good state.

**Recovery process:**

1. **Stop the process.** Ensure no process has the database open.

2. **Restore from checkpoint:**
   ```bash
   # If a hard-link checkpoint was created previously:
   # (Check fjall issue #52 for checkpoint API status)
   rm -rf /path/to/db
   cp -rl /path/to/checkpoint /path/to/db
   ```

3. **Verify restoration.** Open the database and check data integrity:
   ```rust
   let db = Database::builder(path).open()?;
   // Spot-check critical keyspaces
   let count = events.prefix("".as_bytes()).count();
   // Compare against known counts from monitoring
   ```

4. **Accept data loss.** Data written after the checkpoint was created will
   be lost. Document the data loss window.

### Failover Procedure

**Scenario:** Primary instance is down and a standby must take over.

**Recovery process:**

1. **Confirm primary is truly down.** Check process, lock file, and network:
   ```bash
   ssh primary-host "ps aux | grep veloxide"
   ssh primary-host "lsof +D /path/to/db" 2>/dev/null
   ```

2. **Ensure the primary cannot come back.** If the primary might restart
   while the standby is active, both will attempt to write to the same
   storage. This causes corruption. Stop the primary process or fence it:
   ```bash
   ssh primary-host "systemctl stop veloxide"
   # Or fence the host entirely
   ```

3. **Start standby on the database path.** The standby opens the database
   normally. WAL replay handles any uncommitted state:
   ```bash
   veloxide --db-path /path/to/db start
   ```

4. **Verify the standby is healthy.** Check that reads and writes succeed,
  L0 count is not growing unboundedly, and metrics are reporting.

5. **Update routing.** Point clients to the new standby (DNS, load balancer,
  or service discovery update).

### Emergency Diagnostic Commands

```bash
# Database file inventory
find /path/to/db -type f | head -50
du -sh /path/to/db/*

# Check for lock contention
fuser -v /path/to/db 2>&1

# Monitor recovery progress (watch for file changes)
inotifywait -m /path/to/db/journal/

# Disk space check
df -h /path/to/db

# Check for corrupt SSTable files
find /path/to/db -name "*.sst" -size 0
```

### Prevention Checklist

- [ ] Graceful shutdown: trap SIGINT/SIGTERM, explicitly drop `Database`.
  Clean shutdown means empty WAL, so next open is instant.
- [ ] Regular checkpoints for databases with high write rates.
- [ ] Test recovery procedure quarterly with `kill -9` simulation.
- [ ] Monitor WAL file size. Large WALs indicate flush problems and mean
  longer recovery after crash.
- [ ] Document the data loss window (time between last checkpoint and crash)
  as an operational metric.

### Relevant Fjall Version Notes

- V3.0 improved WAL robustness with checksummed journal entries.
- V2.8 added partial WAL recovery (skip corrupt entries instead of failing).
- Checkpoint API (hard-link based, zero-downtime) is tracked in issue #52.
- V3.0 removed the V2.x "thread mode" concept; all operations are
  single-process with exclusive lock.
