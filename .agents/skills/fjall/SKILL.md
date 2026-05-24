---
name: fjall
description: "Expert on the Fjall LSM-tree embedded key-value storage engine for Rust event-sourced orchestrators. Covers Fjall 3.x API (Database, Keyspace, WriteBatch, Snapshot, Guard iterators), LSM-tree internals (block format, compaction, Bloom filters, KV separation), event sourcing patterns (aggregate hydration, snapshots, CQRS), production operations (backup, durability, shutdown), and veloxide-specific integration (vo-storage partitions, BudgetQueues, WriteClass QoS). Use this skill whenever working with vo-storage, event persistence, partition configuration, compaction tuning, read/write performance, or any Fjall-related code in veloxide. Also use for LSM-tree storage questions, embedded databases in Rust, or event sourcing storage design — even if the user doesn't explicitly mention Fjall by name."
---

# Fjall: LSM-Tree Storage Engine for Veloxide

Fjall is a log-structured, embeddable key-value store in 100% safe Rust. It is veloxide's only storage substrate — every workflow event, instance state, timer, and blob passes through Fjall in `vo-storage`.

**V3 Terminology**: `Database` (was "Keyspace" V2), `Keyspace` (was "Partition" V2). Key max 65536 bytes, value max 2^32 bytes. Single process per database (exclusive file lock).

## When This Skill Applies

- Any code in `crates/vo-storage/` — partitions, writes, reads, compaction, snapshots
- Event persistence, replay, hydration, or snapshotting
- Key design for prefix scans, range queries, secondary indices
- Performance tuning: write throughput, read latency, memory, cache sizing
- Compaction strategy (Leveled vs FIFO), Bloom filters, KV separation
- Durability guarantees, WAL configuration, backup/checkpoint
- Debugging cold starts, write stalls, memory spikes, corruption
- Questions about LSM-tree storage engines or event sourcing storage patterns

## Architecture

```
veloxide (vo-storage)
├── Fjall Keyspaces (17 partitions across 3 classes)
│   ├── Hot:  events, instances, timers, dedupe, effects, leases, receipts
│   ├── Cold: snapshots, workflow_versions
│   └── Blob: payload_blobs, blob_records, blob_pack_index
├── fjall::Database (LSM-tree engine)
│   ├── WAL (journal) — sequential durability log
│   ├── Memtables — in-memory sorted write buffers
│   └── SSTables — immutable on-disk sorted files
└── Write Path: Appender → BudgetQueues (3-tier QoS) → OwnedWriteBatch → Fjall
```

For how the LSM-tree works internally, read `references/lsm-internals.md`.

## Quick API Reference

For complete API surface with all types, traits, and method signatures, read `references/api-surface.md`.

```rust
// Database & keyspace
let db = fjall::Database::builder(path).open()?;
let ks = db.keyspace("events", fjall::KeyspaceCreateOptions::default)?; // closure, not value

// CRUD
ks.insert("key", "value")?;
let val: Option<fjall::Slice> = ks.get("key")?;
ks.remove("key")?;

// Atomic cross-keyspace batch (single WAL fsync)
let mut batch = db.batch();
batch.insert(events, "k1", "v1");
batch.insert(index, "k2", "v2");
batch.commit()?;

// Iteration (V3 Guard API — lazy blob loading)
for guard in ks.prefix("prefix") {
    let key = guard.key()?;           // no blob load
    let value = guard.value()?;       // loads blob if separated
    let size = guard.size()?;         // no blob load
}

// Point-in-time snapshot (MVCC, lock-free, cross-keyspace)
let snap = db.snapshot();
let val = snap.get(&ks, "key")?;
for guard in snap.prefix(&ks, "prefix") { /* ... */ }

// Durability
db.persist(fjall::PersistMode::SyncAll)?; // fsync to disk
```

## Key Design for Event Sourcing

Veloxide uses fixed-size big-endian binary keys for lexicographic ordering:

```
events:     [InstanceId_16B | SequenceNumber_8B]              = 24B
instances:  [StatusByte_1B | CreatedAt_8B | InstanceId_16B]   = 25B
timers:     [FireAtMs_8B | InstanceId_16B | TimerId_16B]      = 40B
snapshots:  [InstanceId_16B | SequenceNumber_8B]              = 24B
```

**All integer components use big-endian encoding** (`to_be_bytes()`) so numerical order matches byte order.

For atomic secondary index updates, always use `OwnedWriteBatch` — never write to multiple keyspaces independently.

For event sourcing patterns, aggregate hydration, and CQRS with Fjall, read `references/event-sourcing.md`.

## Veloxide Integration

### Write Path

Writes flow through `Appender` → `BudgetQueues` with three priority tiers (ADR-032):
1. **CriticalControlPlane** (tier 1, cap 1024, never dropped) — event appends, status transitions
2. **OperatorProjection** (tier 2, cap 512) — read model updates
3. **BulkBlob** (tier 3, cap 256) — payload ingestion

Backpressure via `AtomicBool` per class. Metrics: `vo_storage.write_rejected_total`, `vo_storage.queue_depth`.

### Snapshot Policy

`SnapshotPolicy::EveryNEvents(100)` — checkpoint state every 100 events. Hydration: load latest snapshot, then replay only delta events. On snapshot corruption, falls back to full replay from sequence 0.

### FjallEventStore (TODO)

Removed during Fjall V3 migration. Needs reimplementation:
- OCC-based append with sequence validation
- Prefix scan replay via `EventReplayIterator`
- Lineage-aware queries (`InstanceId`, `LineageWide`, `EpochSpecific`)

### Partition Config (Not Yet Wired)

`PartitionConfig` defines class-level settings (bloom bits, flush interval) but `to_fjall_options()` returns defaults — these parameters need wiring into `KeyspaceCreateOptions`.

## Configuration & Tuning

For all tunable parameters with recommended values per workload, read `references/configuration.md`.

**Cache**: Unified cache via `.cache_size(bytes)`. Recommended 20-25% of system RAM.

**Compression**: Per-level policy — skip compression on L0/L1 (max write speed), use LZ4 on deeper levels (historical data). Only `CompressionType::None` and `Lz4` exist in 3.1.4; Zstd is planned.

**Bloom filters**: Essential for point reads. Disable on Lmax via `expect_point_read_hits(true)` to save ~1.25GB per billion keys.

**KV separation**: For large payloads (blobs). Separates values into blob files, LSM-tree stores only pointers. See `references/lsm-internals.md` for details.

**Compaction**: Leveled (default) for event log and indices. FIFO for ephemeral data. Compaction filters (V3.1) for TTL/GDPR. See `references/lsm-internals.md` for trade-offs.

**Durability**: Use `PersistMode::SyncAll` for critical batches. Amortize with `OwnedWriteBatch`.

## Production Operations

For backup, checkpoint, shutdown, and disaster recovery, read `references/operations.md`.

- **Graceful shutdown**: Trap SIGINT/SIGTERM, explicitly drop `Database`. Clean shutdown → millisecond cold start.
- **Backup**: Hard-link based checkpoint (zero-downtime, crash-consistent). API tracked in fjall issue #52.
- **Corruption**: On ungraceful termination, Fjall replays WAL. If corrupt, failover to peer or restore from checkpoint.

## Troubleshooting

For detailed diagnostic procedures, read `references/troubleshooting.md`.

| Symptom | Cause | Fix |
|---------|-------|-----|
| Cold start latency | KV separation blob index load | Increase `cache_size`, warm cache on boot |
| Memory spike (1M+ keys) | KV separation overhead | Cap unified cache, smaller separation threshold |
| Write stalls | L0 > compaction speed | Increase `max_memtable_size`, tune `l0_compaction_trigger` |
| High read latency | Too many L0 segments | Check compaction rate, tune compaction threads |
| Corruption after clear() | Known bug (#277) | Never `clear()` on KV-separated keyspaces |
| Slow prefix scans | Missing prefix locality | Big-endian keys, null-delimited prefixes |

## Reference Files

Read these for deep dives. Each has a table of contents at the top.

- `references/lsm-internals.md` — Block format, SSTable structure, Memtable, Bloom filters, restart intervals, prefix truncation, KV separation, byteview, compaction mechanics
- `references/api-surface.md` — Complete Fjall V3 API: all types, traits, methods, iterators, transactions, snapshots, WriteBatch, compaction filters
- `references/event-sourcing.md` — Event sourcing with Fjall: aggregate patterns, hydration, snapshots, CQRS, evento-fjall, veloxide-specific event store design
- `references/configuration.md` — All tunable parameters: cache, bloom, memtable, compression, compaction, KV separation, with recommended values per workload profile
- `references/operations.md` — Production ops: backup/checkpoint, graceful shutdown, WAL recovery, corruption handling, monitoring, durability trade-offs
- `references/troubleshooting.md` — Diagnostic procedures for cold starts, write stalls, memory issues, corruption, performance regression
- `references/version-history.md` — Feature evolution V1 → V2.0 → V2.6 → V2.8 → V3.0 → V3.1, migration notes, roadmap

## Constraints

1. **Zero external DBs** — Fjall is the only storage. No Redis, no Postgres.
2. **Single binary** — Fjall is embedded, not a separate server process.
3. **One process per database** — V3 exclusive file lock.
4. **Immutable events** — Append-only. Never mutate persisted events.
5. **Group commits** — All writes via `BudgetQueues` / `OwnedWriteBatch`.
6. **Big-endian keys** — All integer key components encoded big-endian.
