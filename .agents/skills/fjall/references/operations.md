# Production Operations for Fjall-Backed Orchestrators

## Table of Contents

1. [Graceful Shutdown](#shutdown)
2. [Backup and Checkpoint](#backup)
3. [WAL Recovery](#wal-recovery)
4. [Corruption Handling](#corruption)
5. [Durability Trade-offs](#durability)
6. [Monitoring and Metrics](#monitoring)
7. [Capacity Planning](#capacity)
8. [Rolling Upgrades](#upgrades)
9. [Multi-Process Safety](#multi-process)
10. [Operational Runbooks](#runbooks)

---

<a id="shutdown"></a>
## 1. Graceful Shutdown

Fjall attempts to persist state when `Database` and `Keyspace` handles are dropped. A clean shutdown is critical for fast recovery.

### Clean Shutdown Sequence

```rust
// 1. Stop accepting new writes
// 2. Drain any pending write queues (BudgetQueues in veloxide)
// 3. Explicitly drop Database — triggers:
//    - WAL fsync
//    - Compaction halt
//    - Metadata markers written to disk
drop(db);
// 4. Process exits
```

### Recovery After Clean Shutdown

On next boot, the engine recognizes clean shutdown markers:
- Skips WAL replay entirely
- Reads structural state directly from segment trailers
- **Cold start: milliseconds** instead of potentially minutes

### Recovery After Dirty Shutdown

If the process is killed (SIGKILL, OOM, power loss):
- Fjall replays WAL segments to reconstruct lost Memtables
- Recovery time proportional to WAL size (max `max_journaling_size`, default 512 MiB)
- Veloxide's `RecoveryThrottleConfig`: `batch_size: 100`, `delay_between_batches_ms: 50`

### Signal Handling

```rust
// Trap SIGINT/SIGTERM in the orchestrator
ctrlc::set_handler(move || {
    // Initiate graceful shutdown
    // Drop Database handle
    // Exit cleanly
})?;
```

---

<a id="backup"></a>
## 2. Backup and Checkpoint

### Hard-Link Based Checkpoint (Proposed in Issue #52)

The planned API:
```rust
Database::backup_to(&self, path: &Path) -> Result<()>
```

**5-step process:**

1. **Lock journal** — Brief exclusive lock on active WAL. Pauses new writes for a fraction of a millisecond.
2. **Fsync and copy** — Active journal and metadata files are synchronously fsynced and copied to backup directory.
3. **Release lock** — Journal lock released. Writes resume with no noticeable latency impact.
4. **Hard-link SSTables** — Immutable SSTables and blob files are hard-linked (not copied) to backup directory. Hard links only update filesystem inode reference counters — no actual data duplication.
5. **Sync** — Final fsync of backup directory.

**Properties**:
- Zero downtime — writes resume after step 3
- Near-instant — hard links take microseconds regardless of data size
- Crash-consistent — captures a point-in-time view
- Space-efficient — no additional disk space until original files are compacted away

### Current Workarounds (Pre-API)

Until the checkpoint API lands in stable Fjall:

1. **Filesystem snapshots** — If using ZFS or Btrfs, take a filesystem-level snapshot of the database directory. This provides the same crash-consistency guarantees.
2. **Copy with database offline** — Stop the orchestrator, copy the entire database directory, restart. Slow but guaranteed consistent.
3. **Application-level export** — Use prefix scans to export events, store in external system. Slower but version-controlled.

### ⚠️ Never Do This

```bash
# NEVER copy an active Fjall database with standard cp
cp -r /path/to/database /backup/database  # CORRUPTION RISK
# The copy may catch the database mid-flush or mid-compaction
# Result: fractured, inconsistent, unrecoverable backup
```

---

<a id="wal-recovery"></a>
## 3. WAL Recovery

### Normal Recovery

On boot after any termination (clean or dirty), Fjall checks the journal:

1. Read journal header to determine last committed SeqNo
2. If clean shutdown markers present → skip replay, load from segment metadata
3. If dirty → replay all unflushed journal entries to reconstruct Memtables
4. Flush reconstructed Memtables to fresh L0 SSTables
5. Database is ready

### Recovery Throttling

Veloxide paces WAL recovery to prevent CPU/disk spikes:

```rust
RecoveryThrottleConfig {
    batch_size: 100,              // events per recovery batch
    delay_between_batches_ms: 50, // pause between batches
}
```

### JournalRecoveryError

If recovery encounters corruption:

```rust
pub enum JournalRecoveryError {
    Io(std::io::Error),      // disk I/O failure
    Corruption,               // data integrity violation
    // ... other variants
}
```

**Response**: The orchestrator supervisor should:
1. Log the specific error variant for diagnostics
2. Halt application panic
3. Failover to replicated peer, or
4. Restore from most recent checkpoint backup

---

<a id="corruption"></a>
## 4. Corruption Handling

### Known Corruption Scenarios

| Scenario | Cause | Prevention |
|----------|-------|------------|
| `clear()` on KV-separated keyspace | Bug #277 | Never call `clear()` on keyspaces with KV separation |
| Power loss during flush | Partial WAL write | Use `PersistMode::SyncAll` |
| Bit rot / hardware degradation | Physical media failure | Periodic checksums, RAID, backups |
| Multi-process access | V3 exclusive lock | Ensure single process per database |

### Checksum Verification

V3 adds default 128-bit xxh3 checksums on:
- Every block read from disk
- Every blob file read

If a checksum mismatch is detected, the read returns an error rather than silently returning corrupted data.

### Recovery Procedures

1. **Localized corruption** — If error affects a single keyspace, other keyspaces may be intact. Attempt read-only access to determine scope.
2. **WAL corruption** — Fjall truncates the journal to the last valid entry. Data after that point is lost but the database remains structurally sound.
3. **Total corruption** — Restore from checkpoint backup or failover to peer.

---

<a id="durability"></a>
## 5. Durability Trade-offs

### PersistMode Spectrum

| Mode | Syscall | Throughput | Data Loss Risk |
|------|---------|------------|----------------|
| `Buffer` | write() only | ~1.2M writes/s | Lost on power failure, OS crash |
| `SyncData` | fdatasync() | Moderate | Very unlikely (no metadata sync) |
| `SyncAll` | fsync() | ~5K writes/s | Zero (hardware durability) |

### fsync Cost Analysis

From VictoriaMetrics benchmarks (similar WAL architecture):
- No fsync: ~1.2M writes/s
- Batched fsync every 200ms: ~1.2M writes/s (amortized)
- fsync per write: ~5K writes/s (240x slower!)

### Amortization Strategy

```rust
// DON'T: fsync after every write
for event in events {
    keyspace.insert(&key, &event)?;
    db.persist(PersistMode::SyncAll)?;  // 240x throughput penalty
}

// DO: batch writes, single fsync
let mut batch = db.batch();
for event in events {
    batch.insert(keyspace, &key, &event);
}
batch.commit()?;  // single fsync for entire batch
db.persist(PersistMode::SyncAll)?;
```

### Veloxide's Approach

Veloxide uses the `BudgetQueues` → `OwnedWriteBatch` pipeline. All writes are accumulated and committed in batches, amortizing the fsync cost. Critical control plane writes (event appends) always use `SyncAll`. Bulk blob writes may use `Buffer` with periodic batch fsync.

---

<a id="monitoring"></a>
## 6. Monitoring and Metrics

### Veloxide Metrics

| Metric | Type | Purpose |
|--------|------|---------|
| `vo_storage.write_rejected_total` | Counter | Writes dropped due to backpressure |
| `vo_storage.queue_depth` | Gauge | Current items in BudgetQueues per WriteClass |

### Fjall-Level Metrics to Monitor

| Metric | How to Check | Alert Threshold |
|--------|-------------|-----------------|
| L0 segment count | `keyspace.approximate_len()` | > 2x `l0_compaction_trigger` |
| Cache hit rate | Compare cached vs uncached read latency | < 80% hit rate |
| Compaction lag | L0 count trend over time | Growing trend |
| Write stall frequency | Write latency p99 | Spikes indicate compaction bottleneck |
| Disk space | Filesystem usage | > 80% capacity |

### Health Check Pattern

```rust
fn storage_health_check(db: &Database) -> HealthStatus {
    let l0_count = events_partition.approximate_len();
    let queue_depth = budget_queues.depth(WriteClass::CriticalControlPlane);

    if l0_count > L0_STALL_THRESHOLD {
        return HealthStatus::Degraded("L0 accumulation exceeds threshold");
    }
    if queue_depth > 800 {
        return HealthStatus::Degraded("Write queue near capacity");
    }
    HealthStatus::Healthy
}
```

---

<a id="capacity"></a>
## 7. Capacity Planning

### Cache Sizing

| Available RAM | Recommended Cache | Notes |
|---------------|-------------------|-------|
| 4 GiB | 800 MiB - 1 GiB | Small deployment |
| 8 GiB | 1.6 - 2 GiB | Standard |
| 16 GiB | 3.2 - 4 GiB | Production |
| 32 GiB | 6.4 - 8 GiB | High-throughput |
| 64+ GiB | 12.8+ GiB | Enterprise |

With KV separation: 100 GB of blob payloads → ~3 MB LSM-tree index. The entire index fits in cache even on modest hardware.

### Disk I/O Planning

- **Write bandwidth**: Each event written to WAL (sequential) + Memtable flush (sequential) + compaction (read + write). Budget ~3x the raw ingestion rate.
- **Compaction bandwidth**: Leveled compaction rewrites 10-30x. With 100 MB/s ingestion, compaction may consume 1-3 GB/s of disk I/O at steady state.
- **NVMe recommendation**: NVMe SSDs sustain 2-7 GB/s sequential writes. Essential for high-throughput deployments.

### File Descriptor Limits

Fjall opens file descriptors for SSTables, blob files, WAL, and metadata. Default `descriptor_table` capacity:
- Linux: 900 FDs
- Windows: 400 FDs
- macOS: 150 FDs

Ensure `ulimit -n` is set appropriately (typically 10K+ for production).

### Veloxide-Specific Sizing

| Partition | Growth Rate | Access Pattern | Recommended |
|-----------|-------------|----------------|-------------|
| events | ~100 events/workflow | Append + prefix scan | 64 MiB memtable, leveled |
| instances | ~1 per workflow | Upsert + prefix scan | Default memtable, leveled |
| snapshots | ~1 per 100 events | Append + latest read | 256 MiB memtable, no bloom |
| payload_blobs | Variable | Content-addressed | KV separation, 1 GiB flush |

---

<a id="upgrades"></a>
## 8. Rolling Upgrades

### MSRV

Fjall V3 requires Rust 1.91+. Veloxide must ensure its toolchain meets this requirement.

### V2 → V3 Migration

1. Use `fjall_v2_v3_migrator` crate to migrate existing data
2. Update all code:
   - `Keyspace` → `Database`
   - `Partition` → `Keyspace`
   - Iterator returns `Guard` not `KvPair`
   - Separate caches → unified `.cache_size()`
3. Test thoroughly — the V3 block format is incompatible with V2

### Fjall Version Pinning

Pin to specific Fjall version in `Cargo.toml`:
```toml
fjall = "=3.1.4"  # pin exact version for storage compatibility
```

The on-disk format changes between major versions. Once a database is opened with V3, it cannot be read by V2.

---

<a id="multi-process"></a>
## 9. Multi-Process Safety

V3 enforces an exclusive file lock on the database directory. This prevents:
- Two orchestrator instances from corrupting the same database
- Accidental parallel access from monitoring tools
- Race conditions from backup scripts

**Implication**: Veloxide's single-binary design aligns perfectly — exactly one process manages the database. For high availability, use application-level replication (primary/failover) rather than shared-disk access.

---

<a id="runbooks"></a>
## 10. Operational Runbooks

### Runbook: Database Won't Start After Crash

**Symptoms**: Orchestrator fails to start, logs show WAL recovery errors.

**Steps**:
1. Check error variant: `JournalRecoveryError::Io` vs `Corruption`
2. If I/O error: check disk health (`smartctl -a /dev/nvme0`), free space
3. If WAL replay is slow (large journal): increase `RecoveryThrottleConfig.batch_size` and reduce `delay_between_batches_ms`
4. If corruption detected: restore from most recent checkpoint backup
5. If no backup available: attempt to open individual keyspaces read-only to salvage data

### Runbook: Compaction Falling Behind

**Symptoms**: Increasing write latency, growing L0 segment count, `approximate_len()` trending up.

**Steps**:
1. Check disk I/O utilization (`iostat -x 1`)
2. If disk is saturated: reduce write rate or upgrade storage
3. Increase `max_memtable_size` to reduce flush frequency
4. Tune `l0_compaction_trigger` — lower value starts compaction sooner
5. For ephemeral data: switch to FIFO compaction (write amp = 1.0)
6. Consider adding compaction filters to proactively remove expired data

### Runbook: Memory Growing Unbounded

**Symptoms**: RSS grows beyond expected cache size, OOM risk.

**Steps**:
1. Check if `cache_size()` is configured (caps the unified cache)
2. Look for long-lived `Slice` values holding blocks alive — copy to `Vec<u8>` if needed
3. Check for long-lived `Snapshot` instances — they prevent version cleanup
4. Verify blob GC is running: check `staleness_threshold` configuration
5. Monitor descriptor table: too many open FDs can indicate compaction issues
