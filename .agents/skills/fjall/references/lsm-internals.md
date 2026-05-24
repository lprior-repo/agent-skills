# LSM-Tree Internals for Fjall

## Table of Contents

1. [Write Path](#write-path)
2. [Read Path](#read-path)
3. [SSTable Structure](#sstable-structure)
4. [Block Format (V3)](#block-format)
5. [InternalValue and MVCC](#internalvalue)
6. [Memtable Design](#memtable)
7. [Bloom Filters](#bloom-filters)
8. [Compaction Mechanics](#compaction)
9. [Range Read Internals](#range-reads)
10. [KV Separation (WiscKey)](#kv-separation)
11. [Byteview Architecture](#byteview)
12. [Partitioned Block Index and TLI](#partitioned-index)

---

<a id="write-path"></a>
## 1. Write Path

Every write to Fjall follows this sequence:

1. **Append to WAL (journal)** — Sequential write to disk for crash recovery. Optional fsync via `PersistMode`. The WAL ensures no data is lost if the process crashes before Memtable flush.

2. **Insert into Memtable** — In-memory sorted structure (BTreeMap in Fjall). The write returns almost instantly since this is a RAM operation. The Memtable holds the most recent data.

3. **Flush to L0 SSTable** — When Memtable size exceeds `max_memtable_size` (default 64 MiB), it is frozen (read-only) and a new Memtable is created. The frozen Memtable is flushed to disk as an immutable L0 SSTable. Writes continue to the new Memtable during flush.

4. **Background compaction** — Compaction threads merge L0 → L1 → L2 → ... through progressively larger levels. This is the core mechanism that maintains read performance.

**Why this suits event sourcing**: Event writes are pure appends — no in-place updates. This maps directly to sequential WAL writes and sorted Memtable inserts. The write path never blocks readers.

### Write Path Configuration

| Parameter | Default | Impact |
|-----------|---------|--------|
| `max_memtable_size` | 64 MiB | Larger = fewer flushes, bigger L0 segments. 128+ MiB for high-throughput |
| `PersistMode` | Buffer | Buffer survives process crash only. SyncAll survives power loss |
| `max_journaling_size` | 512 MiB | Max total WAL size across all keyspaces |

### Write Amplification

Each byte is rewritten multiple times as it cascades through compaction levels:
- Leveled compaction: 10x-30x write amplification
- FIFO compaction: 1.0x (just WAL)
- Bulk loading: 2.0x

**Mitigation**: KV separation for large values, larger memtables to reduce L0 count, tuning `level_fanout`.

---

<a id="read-path"></a>
## 2. Read Path

A point read (get) traverses data sources newest-to-oldest:

1. **Active Memtable** — RAM, instant access. Contains the most recent writes.
2. **Frozen Memtables** — RAM, waiting for flush. Still newer than any SSTable.
3. **L0 SSTables** — On disk, newest files. **May overlap** — all L0 files must be checked (binary search within each).
4. **L1+ SSTables** — On disk, strictly disjoint at each level. At most one segment per level is a candidate for any key.
5. **Bloom filters** — Checked before disk I/O. Can definitively confirm absence, avoiding unnecessary reads.

### Read Amplification

Worst-case I/O for a point read: `L0_segments + (level_count - 1)` disk reads. After L0, only one segment per level is checked.

For **prefix scans** (aggregate hydration), the engine creates a `SegmentMultiReader` per level, initialized with the scan bounds. See [Range Read Internals](#range-reads).

### Cache Behavior

- Block cache holds recently accessed data blocks (4-64 KiB each)
- Unified cache (V2.8+) manages both index blocks and blob payloads
- Cache hit → no disk I/O, sub-microsecond latency
- Cache miss → disk read + decompression + cache insert
- Holding `Slice` values keeps backing block alive in cache

---

<a id="sstable-structure"></a>
## 3. SSTable Structure

Each SSTable is an immutable file with this layout:

```
┌──────────────────────────────────────────────────┐
│ Data Block 1  (4-64 KiB, compressed)             │
│ Data Block 2                                      │
│ Data Block 3                                      │
│ ...                                               │
├──────────────────────────────────────────────────┤
│ Index Blocks  (sparse: first-key → block-offset) │
├──────────────────────────────────────────────────┤
│ TLI Block  (Top Level Index — always in RAM)      │
├──────────────────────────────────────────────────┤
│ Meta Block  (block size, count, key range, etc.)  │
├──────────────────────────────────────────────────┤
│ Trailer  (at fixed negative offset from EOF)      │
│   → pointers to meta, TLI, index blocks           │
└──────────────────────────────────────────────────┘
```

**Key properties**:
- First data block starts at offset 0 (implicit)
- Trailer at fixed negative offset from end of file
- Index blocks enable binary search: TLI → index block → data block
- Blocks are independently compressed
- Files are truly immutable after creation — no reader/writer blocking

**Veloxide impact**: Event keys share `InstanceId` prefix, so prefix truncation within blocks achieves high compression for the key portion. The TLI stays tiny even as the event log grows to terabytes.

---

<a id="block-format"></a>
## 4. Block Format (V3)

V3 completely overhauled the block format for performance:

### Key Improvements over V2

- **Single heap allocation**: Each block is one contiguous buffer (4-64 KiB). No full structural deserialization on read — the block is searched in-place.
- **Embedded sparse index**: Binary search operates directly on the block buffer using restart pointers.
- **Prefix truncation**: Within a block, sorted keys share common prefixes. Only the first key stores the full prefix. Subsequent keys store only the unique suffix. For event keys like `[InstanceId][SeqNo]`, the 16-byte InstanceId is stored once, and only the varying sequence numbers are stored per entry.
- **Restart intervals**: At configurable intervals (default 16 items), the full untruncated key is written. Binary search jumps to the nearest restart point, reconstructs the prefix, then does a localized linear scan.
- **Hash index** (optional): Compact hash map (1 byte per bucket) that bypasses binary search entirely for point reads. Supports up to 254 restart pointers = 4064 items per block.
- **Sequence number shortening**: During compaction of the last level, seqnos are reset to 0 (encoded as 1-byte varint) when no snapshot is affected. Covers ~90% of all KVs.

### Performance Impact

| Metric | V2 | V3 |
|--------|----|----|
| Uncached block read | baseline | 2x-100x faster |
| Memory per small KV | baseline | 2x-5x reduction |
| Index overhead | ~1 byte per restart pointer | u16 for blocks ≤64 KiB, u32 otherwise |

### Veloxide Impact

Event keys (24 bytes) within the same instance share a 16-byte prefix. Prefix truncation means only 8 bytes of unique suffix per key. With restart_interval=16, each restart point stores 24 bytes and 15 intermediate entries store ~8 bytes each. Effective key storage: ~9 bytes per event key instead of 24.

---

<a id="internalvalue"></a>
## 5. InternalValue and MVCC

Every key-value pair is wrapped internally:

```rust
struct InternalValue {
    key: InternalKey,
    value: UserValue,
}

struct InternalKey {
    user_key: UserKey,      // the key you provided
    seqno: SeqNo,           // u64, monotonically increasing timestamp
    value_type: ValueType,  // Insert or Tombstone
}
```

**Sort order**: `(user_key ASC, seqno DESC)` — the most recent version of a key is encountered first during sequential scan.

### MVCC Mechanism

- Each write receives a unique `SeqNo` (monotonically increasing u64)
- Multiple versions of the same logical key coexist — older versions are not overwritten
- Reads at a specific `SeqNo` see a consistent point-in-time snapshot
- Compaction eventually removes obsolete versions when no snapshot holds them
- `Snapshot` type wraps a `SeqNo` to provide a frozen, lock-free read view

### ValueType

- `Insert` — standard data insertion
- `Tombstone` — deletion marker. During compaction, when a tombstone meets an older value for the same key, both are discarded. Tombstones are logically promoted to the deepest level (L6) in V3 to prevent values from resurrecting from incomplete compaction.

---

<a id="memtable"></a>
## 6. Memtable Design

The Memtable is an in-memory sorted write buffer. Fjall uses a `BTreeMap` internally (not a skip list like RocksDB).

### Sizing

| Workload | Recommended `max_memtable_size` |
|----------|-------------------------------|
| Development | 8-16 MiB (default is fine) |
| Production event store | 64-128 MiB |
| High-throughput ingestion | 128-256 MiB |
| Bulk migration | 256+ MiB |

### Flush Behavior

When Memtable exceeds `max_memtable_size`:
1. Current Memtable is frozen (read-only)
2. New Memtable is created for incoming writes
3. Frozen Memtable is flushed to L0 SSTable in background
4. Writes continue to new Memtable during flush — no stall

### L0 Stall Prevention

If L0 accumulates faster than compaction can drain it:
- Fjall uses **intra-L0 compaction** — overlapping L0 segments are merged into larger segments still at L0
- This reduces L0 segment count without waiting for L1 to be available
- If L0 grows beyond a critical threshold, writes are intentionally stalled to prevent memory exhaustion

---

<a id="bloom-filters"></a>
## 7. Bloom Filters

Bloom filters are probabilistic data structures that can definitively confirm the **absence** of a key, allowing the engine to skip unnecessary disk reads.

### Key Properties

- Cannot accelerate range scans — only point reads
- False positive rate (FPR) is configurable via `bits_per_key`
- FPR at 10 bits/key ≈ 0.8%. At 20 bits/key ≈ 0.01%

### Memory Cost

| Items | 10 bits/key | 20 bits/key | FPR |
|-------|------------|------------|-----|
| 100K | 122 KiB | 244 KiB | 0.8% / 0.01% |
| 1M | 1.2 MiB | 2.4 MiB | 0.8% / 0.01% |
| 1B | 1.2 GiB | 2.4 GiB | 0.8% / 0.01% |

### Lmax Optimization

The deepest level (Lmax) contains ~90% of all data. Its Bloom filters consume ~90% of total filter memory. For event sourcing, reads almost always expect to find data (aggregate hydration targets known instance IDs). Setting `expect_point_read_hits(true)` disables Bloom filters on Lmax, saving ~1.25 GiB per billion keys — memory that can go to the block cache instead.

### Partitioned Filters (V3)

Instead of one monolithic filter per SSTable, V3 supports partitioned filters with a Top Level Index (TLI). The TLI is tiny and always pinned in RAM. Individual filter partitions are paged into the cache on demand. This keeps memory usage flat as the event log grows.

**Veloxide recommendation**: Use default bloom settings for hot partitions (events, instances). Disable bloom on Lmax for event partitions since reads always target known instances. Cold and blob partitions have bloom disabled by default (prefix scan heavy).

---

<a id="compaction"></a>
## 8. Compaction Mechanics

Compaction is the background process that merges overlapping SSTables, removes obsolete versions and tombstones, and reorganizes data into deeper levels.

### Leveled Compaction (LCS) — Default

Data cascades through fixed-size levels. L0 accepts new flushes (segments may overlap). L1+ are strictly disjoint (key ranges don't overlap within a level).

**Level sizing**: Level N target = `level_base_size * level_fanout ^ N`

| Parameter | Default | Effect |
|-----------|---------|--------|
| `l0_compaction_trigger` | 4 segments | Lower = faster compaction start, higher = less write amp |
| `level_fanout` | 10 | Higher = flatter tree, faster reads, more write amp |
| `target_size` | 64 MiB | Base segment size for L1+ |

**Trade-offs**:
- Write amplification: 10x-30x (each byte rewritten as it moves through levels)
- Read amplification: Excellent — at most 1 file per level after L0
- Space amplification: ~10% temporary overhead during compaction

### Intra-L0 Compaction

When L1 is blocked by a running compaction, L0 segments consolidate among themselves into larger segments still at L0. This prevents unbounded L0 growth that would destroy read performance.

### L0 Tombstone Promotion (V3)

Tombstones are logically promoted directly to the deepest level (L6) instead of cascading through each intermediate level. This prevents a deleted key from resurrecting if a compaction at a middle level hasn't yet merged the tombstone with the old value.

### FIFO Compaction

All files remain at L0. Oldest segments deleted when total size exceeds threshold. Write amplification = 1.0.

- Good for: ephemeral telemetry, TTL data, heartbeats, short-lived coordination
- Bad for: any data that needs point reads across the full dataset
- RocksDB variant: `allow_compaction = true` enables lightweight intra-FIFO compaction to reduce L0 count

### The Three Amplification Factors

These are in constant tension — improving one degrades the others:

| Strategy | Write Amp | Read Amp | Space Amp |
|----------|-----------|----------|-----------|
| Leveled (LCS) | 10-30x | Low | ~10% |
| Size-Tiered (STCS) | O(log N) | High | Up to 2x |
| FIFO | 1.0x | Very High | Controlled |

### Veloxide Compaction Strategy

- **events, instances** (hot): Leveled — needs bounded read latency for hydration
- **dedupe, timers** (hot): Leveled — point reads and TTL via compaction filters
- **snapshots** (cold): Leveled — infrequent writes, reads target latest only
- **Ephemeral data**: FIFO — if veloxide adds telemetry/heartbeat partitions

---

<a id="range-reads"></a>
## 9. Range Read Internals

Range reads (prefix scans, range queries) use a `SegmentMultiReader` per level, implemented as a double-ended queue.

### Double-Ended Queue Per Level

Each level maintains a queue of candidate segments. Forward iteration consumes from the front; reverse iteration from the back. Once a segment is fully consumed, it is discarded.

Because L1+ segments are strictly disjoint, at most one segment per level is active at any time. This bounds the number of active iterators regardless of total dataset size.

### Monotonic Data Optimization

Event sourcing with monotonically increasing keys (ULIDs, sequence numbers) creates a perfectly disjoint tree structure. Fjall detects this and completely bypasses the standard multi-level search:

- **Normal case**: O(N) initialization across N levels
- **Monotonic case**: O(1) — collapses to a single reader

This is critical for aggregate hydration: regardless of whether the event log holds 1 million or 1 billion events, initializing a prefix scan for a specific instance is instant.

### Veloxide Impact

Event keys `[InstanceId_16B | SequenceNumber_8B]` are naturally monotonic within each instance. Combined with big-endian encoding, this means hydration queries benefit from the O(1) optimization. The engine recognizes the disjoint structure and skips unnecessary level initialization.

---

<a id="kv-separation"></a>
## 10. KV Separation (WiscKey)

Inspired by the WiscKey paper. Large values are separated from the LSM-tree into immutable blob files. The LSM-tree stores only the key plus a lightweight pointer (file offset + length).

### How It Works

1. When a write's value exceeds `separation_threshold`, the value is written to a blob file
2. The LSM-tree entry stores: key → blob_pointer (file offset + length)
3. During compaction, only the tiny pointers are rewritten — the massive blobs stay in place
4. Blob GC: when a blob file exceeds `staleness_threshold` fraction of orphaned payloads, valid blobs are relocated to a fresh file and the old file is deleted

### Configuration

```rust
KvSeparationOptions::default()
    .separation_threshold(4096)        // separate values > 4 KiB
    .file_target_size(128 * 1024 * 1024) // 128 MiB blob files
    .staleness_threshold(0.5)           // GC when 50% stale
    .age_cutoff(0.6)                    // age-based GC trigger
    .compression(CompressionType::Lz4)  // compress blobs
```

### Benefits for Event Store

- **100 GB of blob payloads → ~3 MB LSM-tree index** — entire index fits in RAM
- Write amplification for blobs drops to ~1.0 (written once, never compacted)
- Point reads: 0 disk seeks for index (cached) + 1 direct read for blob payload
- Prefix scans for hydration scan only the tiny index, loading blobs on demand

### Cold Start Issue

With 1M+ keys and KV separation, the first read after restart can cause:
- Latency spike on initial request
- RSS memory spike (~2 GB for a 1.3 GB dataset)
- Subsequent reads become near-instant after blob index is loaded

**Mitigation**: Cap unified cache via `cache_size()`. Warm the cache on boot with a sequential scan.

### Known Bug (#277)

Calling `clear()` on a KV-separated keyspace causes persistent corruption. Error on restart: `"Tried to open a BlobTree, but the existing tree is of type StandardTree."` Status: fixed. **Never use `clear()` on KV-separated keyspaces.**

---

<a id="byteview"></a>
## 11. Byteview Architecture (since V2.6)

The `Slice` type returned by Fjall reads uses `byteview::ByteView` instead of `Arc<[u8]>`.

### Arc Overhead (Before V2.6)

- Fat pointer on stack: 16 bytes (address + length)
- Heap allocation: 16 bytes metadata (strong + weak atomic reference counts)
- Total minimum: 32 bytes per value, independent of payload size
- Creating from `Vec<u8>`: two heap allocations (Vec → Arc conversion)

### ByteView Design

- Fixed struct size: **24 bytes**
- **Inline storage**: Values ≤ 20 bytes stored directly in the struct — no heap allocation. Critical for 16-byte InstanceIds and 24-byte event keys.
- **Heap prefix**: 4-byte prefix of value stored for fast short-circuit comparison without loading full value.
- **Single reference count**: Only strong count (no weak count) — saves 8 bytes per value.
- **Zero-copy slicing**: Can return subslices of existing heap allocations without clone + memcpy.
- **Empty slices**: No allocation at all for unit-type values (secondary index entries with `()` value).

### Performance Impact

For an aggregate hydrated from 10,000 small index entries:
- Arc: 10,000 × 32 bytes = 320 KB overhead
- ByteView: 10,000 × 24 bytes = 240 KB, with most values inline (zero heap allocations)

---

<a id="partitioned-index"></a>
## 12. Partitioned Block Index and TLI

V3 supports two-level index structures for massive SSTables.

### Top Level Index (TLI)

A tiny array of `BlockHandle` structs always pinned in RAM. Each entry points to an index block on disk. Point lookups: 2 binary searches (TLI → index block → data block).

### Demand Paging

Index blocks and filter partitions are paged into the cache on demand. This means:
- Memory usage stays flat as SSTables grow
- Only accessed portions of the index consume cache
- The TLI is the only always-resident structure (very small)

### Veloxide Impact

For the `events` partition with millions of entries across hundreds of SSTables, partitioned indexes prevent the block index from consuming excessive RAM. The TLI stays tiny (a few KB per SSTable), and individual index blocks are loaded only when a hydration query targets that key range.

### Configuration

```rust
KeyspaceCreateOptions::default()
    .index_block_partitioning_policy(PartitioningPolicy::new([
        false,  // L0: not partitioned (small, pinned)
        false,  // L1: not partitioned (small, pinned)
        false,  // L2: not partitioned
        true,   // L3+: partitioned (demand-paged)
    ]))
```

Default: L0/L1/L2 use monolithic indexes (small enough to pin), L3+ use partitioned indexes for memory efficiency.
