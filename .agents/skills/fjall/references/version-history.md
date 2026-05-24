# Fjall Version History and Feature Evolution

## Table of Contents

1. [V1 (May 2024)](#v1)
2. [V2.0 — Key-Value Separation](#v20)
3. [V2.5](#v25)
4. [V2.6 — Byteview Architecture](#v26)
5. [V2.8 — Unified Cache & Bulk Loading](#v28)
6. [V3.0 — Complete Rewrite](#v30)
7. [V3.1 — Compaction Filters](#v31)
8. [Roadmap](#roadmap)
9. [V2 → V3 Migration](#migration)

---

<a id="v1"></a>
## V1 (May 2024)

- Initial release of Fjall
- Basic LSM-tree key-value store in 100% safe Rust
- Single keyspace with partitions
- WAL-based durability
- LZ4 compression (default)

---

<a id="v20"></a>
## V2.0 — Key-Value Separation

- Introduced WiscKey-style key-value separation (blob files)
- Large values stored in immutable blob logs, LSM-tree holds only pointers
- Dramatic reduction in write amplification for large-value workloads
- Partition-based data organization

---

<a id="v25"></a>
## V2.5

- Performance optimizations
- Bug fixes and stability improvements

---

<a id="v26"></a>
## V2.6 — Byteview Architecture

Replaced `Arc<[u8]>` with `byteview::ByteView` for the `Slice` type:

- **Fixed struct size**: 24 bytes (was 32+ with Arc fat pointer + heap metadata)
- **Inline storage**: Values ≤ 20 bytes stored inline — no heap allocation. Critical for UUID-sized keys (16 bytes) and small index entries
- **Heap prefix**: 4-byte prefix stored for short-circuit comparisons without full value scan
- **Single reference count**: Only strong count, no weak count (saves 8 bytes vs Arc)
- **Zero-copy slicing**: Can slice existing heap allocations without clone + memcpy
- **Native io::Read**: Deserialization directly from Slice without intermediate buffer
- **Empty slices**: No heap allocation for unit-type values (secondary index entries)
- Compaction stream optimization bypasses block cache for better throughput
- MSRV: 1.75

**Impact on veloxide**: Event keys (24 bytes) and instance keys (25 bytes) now fit inline or near-inline. Aggregate hydration with thousands of small keys benefits from reduced heap fragmentation.

---

<a id="v28"></a>
## V2.8 — Unified Cache & Bulk Loading

### Unified Cache API

Before V2.8: separate `block_cache` and `blob_cache` configurations with rigid memory partitioning. Result: stranded memory when one cache was full and the other had spare capacity.

V2.8+: Single `.cache_size(bytes)` configuration. Internal unified cache dynamically stores both index blocks and separated blob payloads. Eviction algorithms adapt organically to the workload's actual read distribution.

```rust
// Before V2.8 (deprecated)
.block_cache(Arc::new(BlockCache::with_capacity_bytes(100_000_000)))
.blob_cache(Arc::new(BlobCache::with_capacity_bytes(900_000_000)))

// V2.8+ (recommended)
.cache_size(1_000_000_000) // single unified cache
```

### Bulk Loading

```rust
let stream = (0..1_000_000).map(|x| (encode_key(x), encode_value(x)));
new_keyspace.ingest(stream)?;
```

Requirements: data must be sorted, keyspace must start empty. Skips journal, flushing, compaction entirely. Write amplification of 2 (vs 3 for normal path). Use for: schema migration, database migration, backup restoration.

### Custom Binary Search

Replaced `std::slice::partition_point` with custom implementation to fix performance regression introduced in rustc 1.82. Restored optimal read performance for cached random-key lookups.

---

<a id="v30"></a>
## V3.0 — Complete Rewrite

The largest release in Fjall's history. Complete overhaul of internal format and external API.

### Terminology Change

| V2 Term | V3 Term |
|---------|---------|
| Keyspace | Database |
| Partition | Keyspace |

Migration tool: `fjall_v2_v3_migrator` crate.

### New Block Format

- **Single heap allocation** per block (4-64 KiB), no full structural deserialization on read
- **Embedded sparse index** for binary search within block
- **Prefix truncation**: stores common prefix once, only suffixes for subsequent keys
- **Restart intervals**: default 16 items. Full key at each restart point for binary search
- **Hash index**: optional compact hash map (1 byte/bucket), bypasses binary search for point reads. Supports up to 254 restart pointers = 4064 items
- **Sequence number shortening**: compaction resets seqnos to 0 on last level when no snapshot is affected (covers ~90% of KVs)
- **Performance**: Uncached reads 2x-100x faster than V2. Memory usage reduced 2x-5x for small keys/values

### Guard Iterator API

Iterators now return `Guard` struct for lazy blob loading:

```rust
for guard in keyspace.prefix("prefix") {
    let k = guard.key()?;          // does NOT load blob
    let v = guard.value()?;        // loads blob
    let size = guard.size()?;      // does NOT load blob
    let kv = guard.into_inner()?;  // loads blob, returns (UserKey, UserValue)
    if let Some(kv) = guard.into_inner_if(|key| key.ends_with(b"#html"))? { }
}
```

### Versioning (SuperVersion)

Internal LSM-tree state held in `Version` objects. Compaction/flush creates new Version (copy-on-write). Old versions deleted only when no read snapshot holds them. `Tree` uses single coarse `RwLock` instead of three separate locks — compaction preparation no longer blocks readers.

### Partitioned Filters

Bloom filters cut into partitions with a Top Level Index (TLI). TLI always pinned in memory (very small). Filter partitions paged into cache on demand. L0/L1 filters pinned by default; deeper levels can be paged out.

### L0 Tombstone Promotion

Leveled compaction logically promotes tombstones to L6 immediately, preventing them from getting stuck in middle levels during partial compactions.

### Fluid Per-Level Configuration

Almost all keyspace configs controllable per level:

```rust
KeyspaceCreateOptions::default()
    .data_block_compression_policy(
        CompressionPolicy::new([
            CompressionType::None,              // L0
            CompressionType::None,              // L1
            CompressionType::Lz4,               // L2
            CompressionType::Lz4,                    // L3+ (only None/Lz4 exist in 3.1.4)
        ])
    )
    .expect_point_read_hits(true)  // disable Lmax bloom
    .index_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]))
```

### Other V3 Changes

- **Checksumming**: Default 128-bit xxh3 on block and blob reads from disk
- **Journal compression**: Large values compressed when written to WAL
- **Blob FD caching**: Cached in global `DescriptorTable`, ~15% speedup
- **SFA format**: Simple File Archive for table, blob, version files
- **File locking**: Rust's new file locking API for exclusive multi-process protection
- **MSRV**: 1.90.0 (was 1.75)
- **Binary size**: ~2.2 MB, compile time ~3.5s (vs RocksDB ~12 MB, ~40s)

---

<a id="v31"></a>
## V3.1 — Compaction Filters

Custom logic executed during background compaction for each KV pair.

### API

```rust
use fjall::compaction::filter::{CompactionFilter, Verdict, Context};

struct TtlFilter { cutoff: u64 }

impl CompactionFilter for TtlFilter {
    fn filter_item(&mut self, item: ItemAccessor<'_>, _ctx: &Context) -> lsm_tree::Result<Verdict> {
        if is_expired(item.key(), self.cutoff) {
            Ok(Verdict::Remove)
        } else {
            Ok(Verdict::Keep)
        }
    }
}
```

### Factory Pattern

```rust
struct TtlFilterFactory;

impl Factory for TtlFilterFactory {
    fn make_filter(&self, _ctx: &Context) -> Box<dyn CompactionFilter> {
        Box::new(TtlFilter { cutoff: current_cutoff() })
    }
    fn name(&self) -> &str { "ttl" }
}
```

### Registration

```rust
let db = Database::builder(&path)
    .with_compaction_filter_factories(Arc::new(|keyspace| match keyspace {
        "dedupe" => Some(Arc::new(TtlFilterFactory)),
        _ => None,
    }))
    .open()?;
```

### Constraints

- Compaction timing is non-deterministic — filters run lazily in background
- Application must still handle expired data at read boundary
- Use cases: TTL enforcement, GDPR right-to-be-forgotten, garbage collection
- Tombstone-free deletion — data physically discarded during merge

---

<a id="roadmap"></a>
## Roadmap (from V3 announcement)

| Feature | Status |
|---------|--------|
| Single delete | Planned |
| Prefix filters | Planned |
| Zstd compression | Planned |
| Checkpoint backups | Tracked in issue #52 |
| Merge operator | Planned |
| Range deletions | Planned |
| Encryption | Planned |
| CLI for inspecting/verifying databases | Planned |

---

<a id="migration"></a>
## V2 → V3 Migration

1. Use `fjall_v2_v3_migrator` crate for data migration
2. Update terminology: Keyspace → Database, Partition → Keyspace
3. API changes:
   - Block/blob cache → unified `.cache_size(bytes)`
   - Iterator returns `Guard` not `KvPair` — use `guard.key()?`, `guard.value()?`
   - Keyspace names no longer need to be filename-safe (mapped to integer IDs)
4. MSRV bump: 1.75 → 1.90.0
5. `Slice` type unchanged externally (byteview-backed since V2.6)
6. Single process lock enforced — no multi-process access
