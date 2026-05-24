# Fjall Configuration Reference

All tunable parameters for Fjall 3.x (lsm-tree 3.x) with types, defaults,
recommended ranges, and impact notes. Values sourced from Fjall 3.1.4 and
lsm-tree 3.1.4 source.

## Table of Contents

1. [Quick Config Guide](#1-quick-config-guide)
2. [Database-Level Config](#2-database-level-config)
3. [Keyspace Config](#3-keyspace-config)
4. [Cache Tuning](#4-cache-tuning)
5. [Compression Policy](#5-compression-policy)
6. [Bloom Filter Config](#6-bloom-filter-config)
7. [Memtable Tuning](#7-memtable-tuning)
8. [Compaction Config](#8-compaction-config)
9. [KV Separation Config](#9-kv-separation-config)
10. [Partitioned Index/Filter Config](#10-partitioned-indexfilter-config)
11. [Durability Config](#11-durability-config)
12. [Veloxide Partition Class Defaults](#12-veloxide-partition-class-defaults)

---

## 1. Quick Config Guide

Three recommended configurations for common workloads.

### Development

```rust
use fjall::{Database, KeyspaceCreateOptions, PersistMode};

let db = Database::builder("/tmp/veloxide-dev")
    .cache_size(8 * 1024 * 1024)              // 8 MiB cache (small)
    .worker_threads(2)
    .temporary(true)                           // auto-cleanup on drop
    .open()?;

let events = db.keyspace("events", KeyspaceCreateOptions::default)?;
// No manual fsync needed; journal auto-flushes to OS
```

- Cache: 8 MiB (tiny, fine for dev data sets)
- Durability: `PersistMode::Buffer` implicit (no manual persist calls)
- Compaction: Default Leveled (l0_threshold=4, target_size=64 MiB)
- Compression: Default per-level policy (None/LZ4)
- Bloom: Default 10 bits/key

### Production Event Store

```rust
use fjall::{Database, KeyspaceCreateOptions, PersistMode, compaction::Leveled};
use fjall::config::*;
use std::sync::Arc;

let db = Database::builder("/var/lib/veloxide")
    .cache_size(4 * 1024 * 1024 * 1024)       // 4 GiB unified cache
    .worker_threads(4)
    .max_journaling_size(1024 * 1024 * 1024)   // 1 GiB journal ceiling
    .journal_compression(fjall::CompressionType::Lz4)
    .open()?;

// Hot partition: events, instances, timers
let events = db.keyspace("events", || {
    KeyspaceCreateOptions::default()
        .max_memtable_size(64 * 1024 * 1024)   // 64 MiB memtable
        .filter_policy(FilterPolicy::new([
            FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)),
            FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),
        ]))
        .expect_point_read_hits(true)           // Lmax: skip bloom (~90% filter savings)
        .data_block_compression_policy(CompressionPolicy::new([
            CompressionType::None,               // L0: no compression
            CompressionType::None,               // L1: no compression
            CompressionType::Lz4,                // L2+: LZ4
        ]))
        .index_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]))
        .filter_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]))
        .compaction_strategy(Arc::new(
            Leveled::default()
                .with_l0_threshold(8)
                .with_table_target_size(64 * 1024 * 1024)
        ))
})?;

// Blob partition: payload_blobs (KV-separated for large values)
let blobs = db.keyspace("payload_blobs", || {
    KeyspaceCreateOptions::default()
        .max_memtable_size(256 * 1024 * 1024)   // 256 MiB memtable
        .filter_policy(FilterPolicy::disabled()) // no bloom for blob keys
        .with_kv_separation(Some(
            fjall::KvSeparationOptions::default()
                .separation_threshold(1024)       // 1 KiB threshold
                .file_target_size(128 * 1024 * 1024) // 128 MiB blob files
                .staleness_threshold(0.5)
                .age_cutoff(0.6)
                .compression(fjall::CompressionType::Lz4)
        ))
        .compaction_strategy(Arc::new(
            Leveled::default()
                .with_l0_threshold(4)
                .with_table_target_size(64 * 1024 * 1024)
        ))
})?;

// Writes: batch + fsync for durability
let mut batch = db.batch();
batch.insert(&events, key, value);
batch.commit()?;
db.persist(PersistMode::SyncAll)?;
```

- Cache: 4 GiB (20-25% of 16-20 GiB RAM)
- Durability: `PersistMode::SyncAll` after critical batches
- Compaction: Leveled, L0 threshold 8 for hot, 4 for blob
- KV Separation: blob files 128 MiB, threshold 1 KiB
- Bloom: FPR 0.01% on L0/L1, skip on Lmax

### High-Throughput Ingestion

```rust
use fjall::{Database, KeyspaceCreateOptions, PersistMode, compaction::Fifo};
use std::sync::Arc;

let db = Database::builder("/var/lib/veloxide-ingest")
    .cache_size(8 * 1024 * 1024 * 1024)        // 8 GiB cache
    .worker_threads(8)
    .max_journaling_size(2 * 1024 * 1024 * 1024) // 2 GiB journal
    .max_cached_files(Some(2000))
    .open()?;

// Ephemeral data: FIFO compaction, 10 GiB limit, 1-hour TTL
let ephemeral = db.keyspace("ephemeral", || {
    KeyspaceCreateOptions::default()
        .max_memtable_size(256 * 1024 * 1024)   // 256 MiB memtable
        .filter_policy(FilterPolicy::disabled()) // no bloom needed
        .data_block_compression_policy(CompressionPolicy::all(CompressionType::Lz4))
        .compaction_strategy(Arc::new(
            Fifo::new(10 * 1024 * 1024 * 1024, Some(3600)) // 10 GiB, 1h TTL
        ))
})?;

// Hot events: large memtables to absorb burst writes
let events = db.keyspace("events", || {
    KeyspaceCreateOptions::default()
        .max_memtable_size(256 * 1024 * 1024)   // 256 MiB memtable
        .compaction_strategy(Arc::new(
            fjall::compaction::Leveled::default()
                .with_l0_threshold(16)           // tolerate more L0 segments
                .with_table_target_size(128 * 1024 * 1024)
        ))
})?;

// Batch writes for amortized fsync
let mut batch = db.batch();
for item in burst {
    batch.insert(&events, item.key, item.value);
}
batch.commit()?;
db.persist(PersistMode::SyncData)?;              // fdatasync (skip metadata)
```

- Cache: 8 GiB (large, absorbs read pressure)
- Memtable: 256 MiB (large, absorbs write bursts)
- FIFO: 10 GiB cap, 1-hour TTL for ephemeral data
- Durability: `PersistMode::SyncData` (fdatasync, batched)
- L0 threshold: 16 (relaxed, prevents write stalls during burst)

---

## 2. Database-Level Config

Configured via `Database::builder(path)` which returns a `DatabaseBuilder`.

| Parameter | Builder Method | Type | Default | Recommended Range | Impact |
|-----------|---------------|------|---------|-------------------|--------|
| `path` | `Database::builder(path)` | `&Path` | (required) | Dedicated SSD/NVMe path | Data directory. Must be exclusive to one process (file lock). |
| `cache` | `.cache_size(bytes)` | `u64` (bytes) | 32 MiB | 20-25% of system RAM | Unified block cache shared by all keyspaces. Larger cache reduces disk reads. |
| `journal_compression_type` | `.journal_compression(comp)` | `CompressionType` | `Lz4` (with lz4 feature) / `None` | `Lz4` for production | Compression for large values in WAL. Saves journal disk space. |
| `journal_compression_threshold` | (internal) | `usize` (bytes) | 4096 | 1024-8192 | Values below threshold are stored uncompressed in journal. |
| `manual_journal_persist` | `.manual_journal_persist(flag)` | `bool` | `false` | `false` (auto-flush) / `true` (manual control) | When `true`, disables auto-flush on write. Use with explicit `PersistMode::SyncData`/`SyncAll` calls. |
| `worker_threads` | `.worker_threads(n)` | `usize` | `min(# cores, 4)` | 2-8 | Background compaction/flush threads. More threads = more concurrent compaction. Must be >= 1. |
| `descriptor_table` | `.max_cached_files(n)` | `Option<usize>` | 900 (Linux), 400 (Windows), 150 (macOS) | 500-2000 | LRU cache for file descriptors. More keyspaces = more files = larger table needed. Panics if < 10 or None. |
| `max_journaling_size_in_bytes` | `.max_journaling_size(bytes)` | `u64` (bytes) | 512 MiB | 512 MiB - 4 GiB | Max total WAL size across all keyspaces. Stall writes when exceeded. Must be >= 64 MiB. |
| `max_write_buffer_size_in_bytes` | `.max_write_buffer_size(bytes)` | `Option<u64>` | `None` (unlimited) | `None` or 1-4 GiB | Cap on total memtable memory across all keyspaces. Prevents OOM with many keyspaces. Deprecated/hidden API. Must be >= 1 MiB. |
| `clean_path_on_drop` | `.temporary(flag)` | `bool` | `false` | `true` for tests | Deletes data directory when `Database` is dropped. |
| `compaction_filter_factory_assigner` | `.with_compaction_filter_factories(f)` | `CompactionFilterAssigner` | `None` | Per-keyspace filter factory | Install a factory that assigns compaction filters (TTL, GDPR) to keyspaces by name. |

**MSRV**: Fjall 3.x requires Rust 1.90.0+ (per `Cargo.toml` rust-version).

**File Locking**: V3 enforces exclusive access via file lock. Only one process per database path. Attempting to open a second instance fails.

---

## 3. Keyspace Config

Configured via `KeyspaceCreateOptions` passed to `db.keyspace("name", opts)`.

| Parameter | Builder Method | Type | Default | Recommended Range | Impact |
|-----------|---------------|------|---------|-------------------|--------|
| `max_memtable_size` | `.max_memtable_size(bytes)` | `u64` (bytes) | 64 MiB | 8-256 MiB (per workload) | In-memory write buffer. Larger = fewer flushes, fewer L0 segments. Controls write burst absorption. |
| `level_count` | (fixed) | `u8` | 7 | 7 (hardcoded) | LSM-tree depth. Currently fixed at 7 levels (L0-L6). |
| `data_block_size_policy` | `.data_block_size_policy(policy)` | `BlockSizePolicy` | `all(4096)` | 4-8 KiB (point reads), 16-64 KiB (scans) | Data block size in bytes. Larger blocks improve scan throughput. Smaller blocks reduce point-read latency. Panics if < 1 KiB or > 1 MiB. Set once; not changeable after first write. |
| `data_block_hash_ratio_policy` | `.data_block_hash_ratio_policy(policy)` | `HashRatioPolicy` | `all(0.0)` | 0.0-0.5 | Hash index density in data blocks. > 0 adds a hash map to speed point reads inside blocks at cost of memory/space. Useful for in-memory or heavily cached workloads. |
| `data_block_restart_interval_policy` | `.data_block_restart_interval_policy(policy)` | `RestartIntervalPolicy` | `new([10, 16])` | 10-32 | Restart interval for delta encoding in data blocks. Higher = less space, slower binary search. |
| `index_block_restart_interval_policy` | (fixed) | `RestartIntervalPolicy` | `all(1)` | 1 (fixed) | Restart interval in index blocks. Currently not configurable via public API. |
| `manual_journal_persist` | `.manual_journal_persist(flag)` | `bool` | `false` | Per workload | Keyspace-level override for journal auto-flush. When `true`, caller must call `db.persist()`. |
| `kv_separation_opts` | `.with_kv_separation(opts)` | `Option<KvSeparationOptions>` | `None` | See [Section 9](#9-kv-separation-config) | Enable key-value separation for large values. See dedicated section. |
| `compaction_strategy` | `.compaction_strategy(strategy)` | `Arc<dyn CompactionStrategy>` | `Leveled::default()` | Leveled or FIFO per workload | Compaction algorithm. See [Section 8](#8-compaction-config). |
| `compaction_filter_factory` | (internal) | `Option<Arc<dyn Factory>>` | `None` | Per keyspace | Compaction filter for TTL/expiry. Set via database-level factory assigner. |

---

## 4. Cache Tuning

### Unified Cache API

Fjall V3 uses a single unified block cache shared across all keyspaces within a `Database`.

```rust
// Set at database level; all keyspaces share this cache
let db = Database::builder(path)
    .cache_size(4 * 1024 * 1024 * 1024)  // 4 GiB
    .open()?;
```

| Aspect | Detail |
|--------|--------|
| API | `Database::builder(path).cache_size(bytes)` |
| Scope | Shared by all keyspaces in the database |
| Contents | Data blocks, index blocks, filter blocks, blob references |
| Default | 32 MiB |
| Recommended | 20-25% of system RAM |
| Eviction | LRU |

### Sizing Guidelines

| System RAM | Cache Size | Use Case |
|-----------|------------|----------|
| 4 GiB | 512 MiB - 1 GiB | Small dev / test |
| 8 GiB | 1.5 - 2 GiB | Small production |
| 16 GiB | 3 - 4 GiB | Standard production |
| 32 GiB | 6 - 8 GiB | High-throughput production |
| 64 GiB+ | 12 - 16 GiB | Large data sets, heavy reads |

### Descriptor Table (File Handle Cache)

```rust
let db = Database::builder(path)
    .max_cached_files(Some(2000))
    .open()?;
```

| Platform | Default | Notes |
|----------|---------|-------|
| Linux | 900 | Handles most keyspaces comfortably |
| Windows | 400 | Lower default due to OS limits |
| macOS | 150 | Conservative default |

Each SSTable and blob file needs a file descriptor. With 13 keyspaces and many
compacted levels, 900 is a safe default. Increase to 2000+ for high-throughput
production with many keyspaces.

### Block Pinning

Blocks can be pinned in memory (never evicted from cache). Controlled via
`PinningPolicy`:

```rust
// Default: pin L0 and L1 index blocks, L0 filter blocks
let opts = KeyspaceCreateOptions::default()
    .index_block_pinning_policy(PinningPolicy::new([true, true, false]))
    .filter_block_pinning_policy(PinningPolicy::new([true, false]));
```

| Policy | Constructor | Meaning |
|--------|-------------|---------|
| `PinningPolicy::all(true)` | Pin every level | Best for small data sets that fit in cache |
| `PinningPolicy::all(false)` | Pin nothing | Minimal memory usage |
| `PinningPolicy::new([true, true, false])` | Pin L0, L1 only (default for index) | Good balance; L0/L1 are hot |
| `PinningPolicy::new([true, false])` | Pin L0 only (default for filter) | L0 filters are small and frequently accessed |

**Impact**: Pinned blocks never leave cache. Over-pinning wastes memory.
Under-pinning causes re-reads from disk. Default is good for most workloads.

---

## 5. Compression Policy

Per-level compression controlled via `CompressionPolicy`. The last entry in the
policy array applies to all deeper levels.

### CompressionType

| Variant | Speed | Ratio | Use When |
|---------|-------|-------|----------|
| `CompressionType::None` | Fastest | 1:1 | L0/L1 (write-critical), tiny values |
| `CompressionType::Lz4` | Very fast | ~2:1 | L2+ (general purpose), journal, blobs |

Note: Fjall 3.1.4 / lsm-tree 3.1.4 only ships `None` and `Lz4` (with `lz4`
feature enabled). There is no `Zstd` or `HeavyCompression` variant in this
version. The compression policy array still allows per-level differentiation.

### Policy API

```rust
use fjall::config::CompressionPolicy;

// Different compression per level: [L0, L1, L2+]
let policy = CompressionPolicy::new([
    CompressionType::None,   // L0: max write speed
    CompressionType::None,   // L1: still hot, skip compression
    CompressionType::Lz4,    // L2+: historical data, compress
]);

// Same compression everywhere
let all_lz4 = CompressionPolicy::all(CompressionType::Lz4);

// No compression anywhere
let disabled = CompressionPolicy::disabled(); // = all(None)
```

### Default Compression

With `lz4` feature enabled (the default for veloxide):

| Level | Keyspace Default | Index Block Default |
|-------|-----------------|---------------------|
| L0 | `None` | `None` |
| L1 | `None` | `None` |
| L2+ | `Lz4` | `None` |

Without `lz4` feature:

| Level | Default |
|-------|---------|
| All | `None` |

### Recommended Per-Level Policy

| Workload | L0 | L1 | L2 | L3+ |
|----------|----|----|----|----|
| Event Store (write-heavy) | None | None | Lz4 | Lz4 |
| Blob Storage (large values) | None | Lz4 | Lz4 | Lz4 |
| Read-Heavy (scan-heavy) | None | Lz4 | Lz4 | Lz4 |
| Development | None | None | None | None |

### Index Block Compression

```rust
let opts = KeyspaceCreateOptions::default()
    .index_block_compression_policy(CompressionPolicy::all(CompressionType::None));
```

Default is `None` for all levels. Index blocks are small and compressing them
adds CPU cost for negligible savings. Keep at `None` unless index blocks are
unusually large.

---

## 6. Bloom Filter Config

### FilterPolicy API

```rust
use fjall::config::{FilterPolicy, FilterPolicyEntry, BloomConstructionPolicy, PartitioningPolicy};

// Per-level bloom config: [L0, L1, L2+]
let policy = FilterPolicy::new([
    FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)), // L0: 0.01% FPR
    FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),          // L1+: 10 bits/key
]);

// Same bloom everywhere
let uniform = FilterPolicy::all(FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)));

// Disable filters entirely (not recommended for point reads)
let disabled = FilterPolicy::disabled();
```

### BloomConstructionPolicy

| Variant | Parameter | Meaning |
|---------|-----------|---------|
| `BitsPerKey(f32)` | bits per key | Direct control. 10 bits/key ~ 1% FPR, 20 bits/key ~ 0.001% FPR. |
| `FalsePositiveRate(f32)` | target FPR | Computes bits/key automatically. `0.0001` = 0.01% FPR. |

### FilterPolicyEntry

| Variant | Meaning |
|---------|---------|
| `None` | Skip filter construction for this level |
| `Bloom(policy)` | Build bloom filter with given construction policy |

### Default Filter Policy

```rust
// Default from KeyspaceCreateOptions::default()
FilterPolicy::new([
    FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)), // L0
    FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),          // L1+
])
```

### expect_point_read_hits

```rust
let opts = KeyspaceCreateOptions::default()
    .expect_point_read_hits(true);
```

| Value | Effect | When to Use |
|-------|--------|-------------|
| `false` (default) | Build bloom filters on ALL levels including Lmax | General purpose, when point reads may miss |
| `true` | Skip bloom filter on Lmax (deepest level) | When point reads almost always find the key. Saves ~90% filter space. |

For event stores where keys are almost always present (append-mostly), set to
`true` to save significant disk space. For workloads with many negative lookups
(existence checks, dedup), keep `false`.

### FPR and Size Calculations

Bits per key to FPR relationship (standard bloom filter):

| bits/key | FPR (approx) | Filter Size per 100K keys | Filter Size per 1M keys |
|----------|-------------|--------------------------|------------------------|
| 5 | ~9.2% | 62.5 KiB | 625 KiB |
| 10 | ~0.82% | 125 KiB | 1.25 MiB |
| 15 | ~0.074% | 187.5 KiB | 1.875 MiB |
| 20 | ~0.0067% | 250 KiB | 2.5 MiB |
| 23 | ~0.0011% | 287.5 KiB | 2.875 MiB |
| 30 | ~0.0000093% | 375 KiB | 3.75 MiB |

For `FalsePositiveRate(0.0001)` (0.01% FPR), approximately 15-16 bits per key:
- 100K keys: ~188 KiB filter
- 1M keys: ~1.88 MiB filter
- 100M keys: ~188 MiB filter

### Partitioned Filters

Controlled via `PartitioningPolicy` (alias for `PinningPolicy`):

```rust
// Default: partition filters starting at L3+
let opts = KeyspaceCreateOptions::default()
    .filter_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]));
```

Partitioned filters split the bloom into smaller blocks, allowing demand-paging
of filter data instead of loading the entire filter into memory. Beneficial for
deep levels with large filter sizes.

---

## 7. Memtable Tuning

### max_memtable_size

```rust
let opts = KeyspaceCreateOptions::default()
    .max_memtable_size(64 * 1024 * 1024);  // 64 MiB
```

| Workload | Recommended | Rationale |
|----------|-------------|-----------|
| Hot control plane (events, instances) | 64 MiB (default) | Balanced write absorption vs. memory |
| High-throughput ingestion | 128-256 MiB | Absorb larger write bursts before flush |
| Blob storage | 256-1024 MiB | Fewer, larger flushes for big values |
| Development/testing | 8-16 MiB | Lower memory footprint |

### Flush Trigger Behavior

When a keyspace's active memtable reaches `max_memtable_size`:

1. The memtable is frozen (becomes immutable).
2. A new active memtable is created.
3. The frozen memtable is flushed to an L0 SSTable in the background.
4. The frozen memtable is released.

If L0 accumulates too many SSTables (exceeding `l0_compaction_trigger`),
writes may stall until compaction catches up.

### L0 Stall Prevention

| Symptom | Cause | Fix |
|---------|-------|-----|
| Write stalls / timeouts | L0 table count exceeds threshold | Increase `max_memtable_size` or `l0_threshold` |
| Memory pressure | Too many frozen memtables | Reduce `max_memtable_size` per keyspace |
| Slow compaction | Not enough worker threads | Increase `worker_threads` on `Database::builder` |

### max_write_buffer_size (Database-Level)

```rust
// Cap total memtable memory across ALL keyspaces (hidden/deprecated API)
let db = Database::builder(path)
    .max_write_buffer_size(Some(4 * 1024 * 1024 * 1024))  // 4 GiB total
    .open()?;
```

This caps the combined memory of all memtables across all keyspaces. When the
total exceeds this limit, writes stall until memtables flush. Useful when
running many keyspaces on memory-constrained systems. Currently a hidden/deprecated API.

---

## 8. Compaction Config

### Leveled Compaction (Default, Recommended)

```rust
use fjall::compaction::Leveled;
use std::sync::Arc;

let strategy = Leveled::default()
    .with_l0_threshold(8)                        // trigger L0->L1 at 8 segments
    .with_table_target_size(64 * 1024 * 1024)    // 64 MiB target SSTable size
    .with_level_ratio_policy(vec![10.0]);         // 10x growth per level
```

| Parameter | Method | Type | Default | Recommended Range | Impact |
|-----------|--------|------|---------|-------------------|--------|
| `l0_threshold` | `.with_l0_threshold(n)` | `u8` | 4 | 4-16 | Number of L0 tables that triggers L0->L1 compaction. Higher = more write tolerance, slower reads. Same as `level0_file_num_compaction_trigger` in RocksDB. |
| `target_size` | `.with_table_target_size(bytes)` | `u64` | 64 MiB | 16-256 MiB | Target SSTable file size on disk. Larger files = fewer files = less overhead. Same as `target_file_size_base` in RocksDB. |
| `level_ratio_policy` | `.with_level_ratio_policy(vec)` | `Vec<f32>` | `vec![10.0]` | 8.0-16.0 | Size ratio between levels. L1 = target_size * l0_threshold. L2 = L1 * ratio. Higher = fewer levels used, more compaction per level. Same as `max_bytes_for_level_multiplier` in RocksDB. |

### Level Size Calculation

```
L1_size = target_size * l0_threshold
L2_size = L1_size * ratio
L3_size = L2_size * ratio
...
```

With defaults (target=64 MiB, threshold=4, ratio=10):

| Level | Max Size |
|-------|----------|
| L0 | 4 * memtable_size (variable) |
| L1 | 256 MiB (64 * 4) |
| L2 | 2.5 GiB |
| L3 | 25 GiB |
| L4 | 250 GiB |
| L5 | 2.5 TiB |
| L6 (Lmax) | 25 TiB |

### FIFO Compaction (Ephemeral Data Only)

```rust
use fjall::compaction::Fifo;
use std::sync::Arc;

let strategy = Fifo::new(
    10 * 1024 * 1024 * 1024,  // 10 GiB size limit
    Some(3600),                // 1 hour TTL (optional)
);
```

| Parameter | Type | Default | Recommended | Impact |
|-----------|------|---------|-------------|--------|
| `limit` | `u64` (bytes) | (required) | 1-100 GiB | Data set size cap. Oldest tables deleted when exceeded. Includes blob file bytes. |
| `ttl_seconds` | `Option<u64>` | `None` | `None` or 300-86400 | Lazy TTL. Tables older than TTL are dropped during compaction. |

**Cautions for FIFO:**
- Only use for append-only, time-ordered data (logs, metrics, ephemeral state).
- Never use for data that requires updates or deletes.
- Key order must be strictly monotonically increasing or decreasing.
- FIFO never merges; it only drops old tables.

### Intra-L0 Compaction

Leveled compaction performs intra-L0 merging when L0 has too many segments.
This is automatic and not directly configurable. The `l0_threshold` parameter
controls when L0->L1 compaction triggers, which indirectly controls intra-L0
behavior.

### Compaction Filters (V3.1)

```rust
let db = Database::builder(path)
    .with_compaction_filter_factories(Arc::new(|keyspace_name: &str| {
        match keyspace_name {
            "dedupe" => Some(Arc::new(MyTtlFilter::new(3600))),
            _ => None,
        }
    }))
    .open()?;
```

Compaction filters run during compaction to drop or modify entries. Use for:
- TTL-based expiry (drop entries older than N seconds)
- GDPR compliance (drop entries matching a pattern)
- Custom garbage collection logic

---

## 9. KV Separation Config

Key-value separation moves large values into separate blob files, leaving only
pointers in the LSM-tree. Reduces write amplification for large-value workloads.

```rust
use fjall::KvSeparationOptions;

let opts = KeyspaceCreateOptions::default()
    .with_kv_separation(Some(
        KvSeparationOptions::default()
            .separation_threshold(1024)              // 1 KiB
            .file_target_size(128 * 1024 * 1024)     // 128 MiB
            .staleness_threshold(0.5)                 // 50% stale
            .age_cutoff(0.6)                          // 60% age
            .compression(fjall::CompressionType::Lz4)
    ));
```

### KvSeparationOptions Parameters

| Parameter | Method | Type | Default | Recommended Range | Impact |
|-----------|--------|------|---------|-------------------|--------|
| `separation_threshold` | `.separation_threshold(bytes)` | `u32` (bytes) | 1024 (1 KiB) | 256-4096 | Values >= this size are separated into blob files. Smaller = more separation = less compaction overhead but slower reads. |
| `file_target_size` | `.file_target_size(bytes)` | `u64` (bytes) | 64 MiB | 64-256 MiB | Target blob file size. Smaller = more granular GC. Larger = fewer files, less overhead. |
| `staleness_threshold` | `.staleness_threshold(ratio)` | `f32` | 0.25 (25%) | 0.2-0.5 | Blob file GC trigger. When stale (overwritten/deleted) entries exceed this percentage, the file is rewritten. Higher = less GC write overhead, more space waste. |
| `age_cutoff` | `.age_cutoff(ratio)` | `f32` | 0.25 (25%) | 0.2-0.6 | Age-based GC threshold. Blob files with a high ratio of old entries are candidates for GC. |
| `compression` | `.compression(comp)` | `CompressionType` | `Lz4` (with feature) | `Lz4` or `None` | Compression for blob file contents. |

### When to Use KV Separation

| Use Case | Threshold | File Size | Why |
|----------|-----------|-----------|-----|
| Event payloads (>1 KiB) | 1024 | 128 MiB | Reduces compaction of large event bodies |
| Encrypted blobs (>4 KiB) | 4096 | 128-256 MiB | Prevents rewriting blobs during compaction |
| Small metadata (<1 KiB) | (disable) | N/A | Not worth the indirection overhead |
| Mixed hot/cold | 1024 | 64 MiB | Separates cold blobs from hot metadata |

**Important**: Never call `clear()` on a KV-separated keyspace (Fjall bug #277).
Doing so corrupts the blob index.

---

## 10. Partitioned Index/Filter Config

### PartitioningPolicy

`PartitioningPolicy` is a type alias for `PinningPolicy` (both are `Vec<bool>`
under the hood, where the last element applies to all deeper levels).

```rust
use fjall::config::PartitioningPolicy;

// Default: partition starting at L3+
// [L0=false, L1=false, L2=false, L3+=true]
PartitioningPolicy::new([false, false, false, true])

// Never partition
PartitioningPolicy::all(false)

// Always partition
PartitioningPolicy::all(true)
```

### Index Block Partitioning

```rust
let opts = KeyspaceCreateOptions::default()
    .index_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]));
```

| Setting | Effect | When to Use |
|---------|--------|-------------|
| `all(false)` | Full index loaded as single block | Small data sets where index fits in cache |
| `new([false, false, false, true])` | Partitioned from L3+ (default) | Standard production. TLI in memory, deep levels demand-paged. |
| `all(true)` | All levels partitioned | Very large data sets where even L0/L1 index blocks are large |

### Filter Block Partitioning

```rust
let opts = KeyspaceCreateOptions::default()
    .filter_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]));
```

Same pattern as index partitioning. Partitioned filters allow demand-paging
of bloom filter data for deep levels, reducing memory pressure.

### Top-Level Index (TLI)

The top-level index block of partitioned indexes is always kept in memory
(pinned). This enables O(1) lookup to find the correct partition without
scanning. The TLI is typically very small (a few KiB) even for large data sets.

### Pinning Policies

```rust
// Index blocks: pin L0 and L1 (default)
let opts = KeyspaceCreateOptions::default()
    .index_block_pinning_policy(PinningPolicy::new([true, true, false]));

// Filter blocks: pin L0 only (default)
let opts = KeyspaceCreateOptions::default()
    .filter_block_pinning_policy(PinningPolicy::new([true, false]));
```

Pinned blocks are never evicted from the cache. L0/L1 index blocks are hot and
small, so pinning them avoids re-reads. L0 filter blocks are also small and
benefit from pinning.

---

## 11. Durability Config

### PersistMode

```rust
// After writing a batch or single operation:
db.persist(PersistMode::SyncAll)?;
```

| Mode | syscall | Guarantee | Performance | When to Use |
|------|---------|-----------|-------------|-------------|
| `Buffer` | (none) | Data in OS page cache. Survives app crash, NOT power loss. | Fastest | Development, non-critical data, batches that will be followed by SyncAll |
| `SyncData` | `fdatasync` | Data on disk. No metadata sync. Survives power loss on most FS. | Fast | Production batches where metadata timestamp is not critical |
| `SyncAll` | `fsync` | Data + metadata on disk. Full durability. | Slowest | Critical control-plane writes, after large batches, ACID transactions |

### Auto-Flush Behavior

By default (`manual_journal_persist = false`), each write or batch commit
automatically flushes to the OS page cache (`Buffer`-equivalent). This means:

- Application crash: data survives
- OS crash / power loss: data may be lost

For full durability, call `db.persist(PersistMode::SyncAll)` after critical
writes.

### Manual Journal Persist

```rust
// Disable auto-flush for maximum write throughput
let db = Database::builder(path)
    .manual_journal_persist(true)
    .open()?;

// ... writes happen ...

// Manually persist when needed
db.persist(PersistMode::SyncAll)?;
```

When `manual_journal_persist = true`, the journal writer does not flush to the
OS after each write. The caller is responsible for calling `db.persist()` at
appropriate intervals. This amortizes fsync cost across many writes.

### Amortizing fsync via WriteBatch

```rust
// BAD: fsync per write (slow)
for item in items {
    events.insert(item.key, item.value)?;
    db.persist(PersistMode::SyncAll)?;  // fsync per item!
}

// GOOD: batch + single fsync (fast)
let mut batch = db.batch();
for item in items {
    batch.insert(&events, item.key, item.value);
}
batch.commit()?;                  // single WAL write
db.persist(PersistMode::SyncAll)?; // single fsync for all items
```

### Journal (WAL) Configuration

| Parameter | Default | Notes |
|-----------|---------|-------|
| Pre-allocated size | 64 MiB | Journal files are pre-allocated to reduce fragmentation |
| Write buffer | 8 KiB | `BufWriter` capacity for journal IO |
| Max total journal size | 512 MiB | `max_journaling_size`. Writes stall when exceeded. |
| Compression | Lz4 (with feature) | Applied to values >= compression_threshold (4096 bytes) |
| Rotation | Automatic | Journal rotates (fsync + create new) during flush |

---

## 12. Veloxide Partition Class Defaults

Veloxide defines three partition classes in `vo-storage::partitions`. Each class
has tuned `PartitionConfig` values. Note: `to_fjall_options()` currently returns
defaults -- these parameters need wiring into `KeyspaceCreateOptions`.

### Hot Class (Control Plane)

**Partitions**: events, instances, timers, dedupe, effects, leases, receipts

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `compaction_enabled` | `true` | Event log needs compaction for read performance |
| `bloom_filter_bits_per_key` | 10 | Point reads are frequent (event lookup, dedup check) |
| `flush_interval_bytes` | 64 MiB | Standard memtable size, balanced write absorption |

### Cold Class (Historical)

**Partitions**: snapshots, workflow_versions

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `compaction_enabled` | `true` | Snapshots need compaction for space reclamation |
| `bloom_filter_bits_per_key` | 0 | Scans dominate; point reads are rare |
| `flush_interval_bytes` | 256 MiB | Larger memtables for bulk snapshot writes |

### Blob Class (Large Payloads)

**Partitions**: payload_blobs, blob_records, blob_pack_index

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `compaction_enabled` | `true` | Space reclamation for overwritten blobs |
| `bloom_filter_bits_per_key` | 0 | Content-addressed; existence checks use index partitions |
| `flush_interval_bytes` | 1 GiB | Very large memtables for bulk blob ingestion |

### Target Fjall Options (When Wired)

When `to_fjall_options()` is properly wired, the intended configuration is:

```rust
// Hot partition (events, instances, timers, dedupe, effects, leases, receipts)
fn hot_fjall_options() -> KeyspaceCreateOptions {
    KeyspaceCreateOptions::default()
        .max_memtable_size(64 * 1024 * 1024)
        .filter_policy(FilterPolicy::new([
            FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)),
            FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),
        ]))
        .expect_point_read_hits(true)
        .data_block_compression_policy(CompressionPolicy::new([
            CompressionType::None,
            CompressionType::None,
            CompressionType::Lz4,
        ]))
        .compaction_strategy(Arc::new(
            Leveled::default()
                .with_l0_threshold(8)
                .with_table_target_size(64 * 1024 * 1024)
        ))
}

// Cold partition (snapshots, workflow_versions)
fn cold_fjall_options() -> KeyspaceCreateOptions {
    KeyspaceCreateOptions::default()
        .max_memtable_size(256 * 1024 * 1024)
        .filter_policy(FilterPolicy::disabled())
        .data_block_compression_policy(CompressionPolicy::all(CompressionType::Lz4))
        .compaction_strategy(Arc::new(
            Leveled::default()
                .with_l0_threshold(4)
                .with_table_target_size(128 * 1024 * 1024)
        ))
}

// Blob partition (payload_blobs, blob_records, blob_pack_index)
fn blob_fjall_options() -> KeyspaceCreateOptions {
    KeyspaceCreateOptions::default()
        .max_memtable_size(1024 * 1024 * 1024)  // 1 GiB
        .filter_policy(FilterPolicy::disabled())
        .with_kv_separation(Some(
            KvSeparationOptions::default()
                .separation_threshold(1024)
                .file_target_size(128 * 1024 * 1024)
                .staleness_threshold(0.5)
                .age_cutoff(0.6)
                .compression(CompressionType::Lz4)
        ))
        .compaction_strategy(Arc::new(
            Leveled::default()
                .with_l0_threshold(4)
                .with_table_target_size(64 * 1024 * 1024)
        ))
}
```
