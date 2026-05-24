# Event Sourcing with Fjall -- Veloxide Reference

## Table of Contents

1. [Event Sourcing Fundamentals](#1-event-sourcing-fundamentals)
2. [Event Store Design with Fjall](#2-event-store-design-with-fjall)
3. [Aggregate Hydration](#3-aggregate-hydration)
4. [Snapshot Strategy](#4-snapshot-strategy)
5. [CQRS Read Models](#5-cqrs-read-models)
6. [Optimistic Concurrency Control (OCC)](#6-optimistic-concurrency-control-occ)
7. [Key Encoding for Event Streams](#7-key-encoding-for-event-streams)
8. [Event Schema Evolution](#8-event-schema-evolution)
9. [Veloxide-Specific Patterns](#9-veloxide-specific-patterns)
10. [Cross-Partition Atomicity](#10-cross-partition-atomicity)

---

## 1. Event Sourcing Fundamentals

### Core Principles

Event sourcing stores **every state change as an immutable event** rather than
mutating current state in place. The current state is always derived by folding
over the complete event history.

**Three invariants:**

1. **Immutability** -- events are never modified or deleted after being
   committed. Corrections are appended as new events (compensating actions).
2. **Append-only** -- the event store only supports `append` operations. There
   is no `update` or `delete` in the normal write path.
3. **State as fold** -- current aggregate state is a left-fold over all events
   for a given stream, starting from the initial (default) state.

```rust
/// The fundamental fold: state = events.fold(initial_state, apply)
fn hydrate_aggregate<S, E>(events: impl Iterator<Item = E>, initial: S) -> S
where
    S: Aggregate<E>,
{
    events.fold(initial, |state, event| state.apply(event))
}

/// An aggregate knows how to apply a single event to produce new state.
pub trait Aggregate<E>: Sized {
    fn apply(self, event: E) -> Self;
}
```

### Why This Matters for Veloxide

Veloxide uses event sourcing for workflow instances. Each `InstanceId` has its
own event stream stored in the `events` Fjall partition. When a workflow actor
is rehydrated (after hibernation or crash), the engine replays events to
reconstruct the workflow's state machine position. This provides:

- **Complete audit trail** -- every state transition is recorded.
- **Temporal queries** -- the engine can reconstruct state at any point in time.
- **Crash recovery** -- replaying events rebuilds state without data loss.

---

## 2. Event Store Design with Fjall

### Partition Layout (Veloxide)

Veloxide uses a dedicated Fjall keyspace per concern. The partition layout
is defined in `vo-storage/src/partitions.rs`:

| Partition | Purpose | Key Pattern | Class |
|-----------|---------|-------------|-------|
| `events` | Event log + state transitions | `[instance_id(16)][sequence_u64_be]` | Hot |
| `instances` | Materialized instance index | `[status(1)][created_at(8)][instance_id(16)]` | Hot |
| `snapshots` | Periodic replay checkpoints | `[instance_id(16)][sequence_u64_be]` | Cold |
| `timers` | Durable wake-up schedule | `[timestamp(8)][iid_len(2)][instance_id]` | Hot |
| `effects` | Effect journal (prepare/commit) | `[instance_id(16)][sequence(8)][0xFF]` | Hot |
| `dedupe` | Exactly-once ingress | `[idempotency_key]` | Hot |
| `leases` | Monotonic fence tokens | `[instance_id(16)][step_id...]` | Hot |
| `receipts` | Execution receipts | `[effect_id]` | Hot |
| `lineage` | Continue-as-new routing | `[lineage_id]` | Hot |
| `workflow_versions` | WorkflowSpec by hash | `[hash]` | Cold |
| `payload_blobs` | Encrypted payload blobs | `[content_addr]` | Blob |

### Hot/Cold/Blob Classification (Veloxide abstraction)

`PartitionConfig` is a Veloxide wrapper (vo-storage), not a Fjall type. It maps to `fjall::KeyspaceCreateOptions`:

```rust
// Hot: bloom filter 10 bits/key, 64MB flush → translates to KeyspaceCreateOptions
// Cold: no bloom filter, 256MB flush (snapshots, workflow_versions)
// Blob: no bloom filter, 1GB flush (payload_blobs, blob_records)
// See configuration.md §12 for the actual Fjall options mapping.
```

### Opening the Database

```rust
use fjall::Database;

let db = Database::builder("/path/to/storage")
    .open()
    .map_err(|e| StorageError::InvalidPath { reason: e.to_string() })?;

// Open a specific partition with appropriate config
let events_partition = db.keyspace("events", || {
    fjall::KeyspaceCreateOptions::default()
})?;
```

### Event Schema

Each event is stored as an `EventEnvelope` (defined in `vo-types`):

```rust
pub struct EventEnvelope {
    pub schema_version: u16,
    pub instance_id: String,
    pub sequence: u64,
    pub timestamp_ms: u64,
    pub payload: serde_json::Value,
    pub metadata: EventMetadata,
}
```

### Write Path

Events are appended through the `EventStore` trait with OCC validation:

```rust
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to an instance stream. Returns the final sequence number.
    /// Rejects if sequence numbers don't follow the current stream position.
    async fn append(
        &self,
        instance_id: &InstanceId,
        events: Vec<EventEnvelope>,
    ) -> Result<u64, EventStoreError>;

    /// Get the current sequence number for an instance (0 if no events).
    async fn get_sequence(
        &self,
        instance_id: &InstanceId,
    ) -> Result<u64, EventStoreError>;
}
```

---

## 3. Aggregate Hydration

### Prefix Scan Pattern

Fjall stores keys in sorted order. For a given `InstanceId`, all events are
stored under a 16-byte prefix. A prefix scan returns events in sequence order
because the sequence number is encoded as big-endian (sorted ascending).

```rust
use crate::key_encoding::{encode_event_key, get_event_key_prefix};
use vo_types::{InstanceId, SequenceNumber};

/// Replay all events for an instance from the events partition.
fn replay_events(
    partition: &fjall::Keyspace,
    instance_id: &InstanceId,
) -> impl Iterator<Item = Result<EventEnvelope, StorageError>> {
    let prefix = instance_id.to_bytes().unwrap_or([0u8; 16]);
    partition.prefix(prefix).map(|guard| {
        let (key, value) = guard.into_inner().map_err(|_| StorageError::FjallError)?;
        let envelope = EventEnvelope::from_bytes(&value)
            .map_err(|_| StorageError::CorruptEventPayload)?;
        Ok(envelope)
    })
}
```

### Fold-Based Reconstruction

```rust
/// Hydrate an aggregate by folding events from the store.
fn hydrate<S: Aggregate<EventEnvelope>>(
    partition: &fjall::Keyspace,
    instance_id: &InstanceId,
    initial_state: S,
) -> Result<S, StorageError> {
    let events = replay_events(partition, instance_id);
    events.fold(Ok(initial_state), |state_result, event_result| {
        let state = state_result?;
        let event = event_result?;
        Ok(state.apply(event))
    })
}
```

### Sequence Validation with IteratorState

Veloxide's `IteratorState` validates that events arrive in strict sequential
order during replay. It tracks the expected next sequence number and detects
gaps:

```rust
pub struct IteratorState {
    expected: Option<u64>,
    started: bool,
}

impl IteratorState {
    pub const fn new() -> Self {
        Self { expected: None, started: false }
    }

    /// Validate sequence continuity. Returns the event on success,
    /// or StorageError::SequenceGap if a gap is detected.
    pub fn advance(
        &mut self,
        found: u64,
        record: EventEnvelope,
    ) -> Option<Result<EventEnvelope, StorageError>> {
        if found == 0 {
            return Some(Err(StorageError::InvalidArgument));
        }
        if !self.started {
            self.started = true;
            self.expected = found.checked_add(1);
            return Some(Ok(record));
        }
        match self.expected {
            Some(expected) if found != expected => {
                Some(Err(StorageError::SequenceGap))
            }
            Some(expected) => {
                self.expected = expected.checked_add(1);
                Some(Ok(record))
            }
            None => Some(Err(StorageError::SequenceGap)),
        }
    }
}
```

### EventReplayIterator

The production iterator wraps a Fjall range scan with sequence validation:

```rust
pub struct EventReplayIterator {
    state: IteratorState,
    inner: Option<Box<dyn DoubleEndedIterator<Item = fjall::Guard>>>,
    init_error: Option<StorageError>,
}

// Construction: range scan from sequence 1 to u64::MAX under the instance prefix
pub fn replay_events(
    keyspace: &fjall::Database,
    instance_id: &InstanceId,
) -> EventReplayIterator {
    let prefix = prefix_generator(instance_id)
        .map_err(EventReplayIterator::error)?;

    let partition = keyspace.keyspace("events", || {
        fjall::KeyspaceCreateOptions::default()
    }).map_err(EventReplayIterator::error)?;

    let mut start = prefix.clone();
    start.extend_from_slice(&1u64.to_be_bytes()); // sequence 1
    let mut end = prefix;
    end.extend_from_slice(&u64::MAX.to_be_bytes());

    EventReplayIterator {
        state: IteratorState::new(),
        inner: Some(Box::new(partition.range(start..=end))),
        init_error: None,
    }
}
```

---

## 4. Snapshot Strategy

### Every N Events

Veloxide uses a configurable snapshot policy. The default takes a snapshot every
100 events:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPolicy {
    EveryNEvents(u64),
    Disabled,
}

impl Default for SnapshotPolicy {
    fn default() -> Self {
        Self::EveryNEvents(100)
    }
}

impl SnapshotPolicy {
    pub const fn should_snapshot(&self, current_sequence: u64) -> bool {
        match self {
            Self::EveryNEvents(n) => {
                current_sequence > 0 && current_sequence.is_multiple_of(*n)
            }
            Self::Disabled => false,
        }
    }
}
```

### Snapshot Keyspace

Snapshots are stored in a separate `snapshots` partition (Cold class). The key
format mirrors event keys:

```
[InstanceId 16 bytes | SequenceNumber 8 bytes] = 24 bytes, big-endian
```

```rust
/// Encode a snapshot key: [instance_id(16) | sequence_u64_be(8)]
pub fn encode_snapshot_key(
    instance_id: &InstanceId,
    sequence: u64,
) -> Result<[u8; 24], StorageError> {
    let id_bytes = instance_id.to_bytes().map_err(|_| StorageError::CorruptKey)?;
    let seq_bytes = sequence.to_be_bytes();
    let mut key = [0u8; 24];
    key[..16].copy_from_slice(&id_bytes);
    key[16..].copy_from_slice(&seq_bytes);
    Ok(key)
}
```

### Snapshot Value Format

Snapshots store a `SnapshotHeader` (with CRC32 checksum) followed by the
serialized `InstanceState`:

```
[header_json_bytes | 0x7C (pipe) | state_json_bytes]
```

```rust
pub struct SnapshotHeader {
    pub version: u16,           // schema version (currently 1)
    pub sequence_number: u64,   // sequence at which snapshot was taken
    pub instance_id: InstanceId,
    pub checksum: u32,          // CRC32 of the state JSON
}
```

### Delta Replay

On hydration, the `ReplayFetcher` loads the latest snapshot first, then only
replays events after the snapshot sequence:

```rust
pub struct ReplayFetcher<S: SnapshotReader, E: EventStore> {
    snapshot_store: S,
    event_store: E,
}

impl<S: SnapshotReader, E: EventStore> ReplayFetcher<S, E> {
    pub fn fetch_snapshot_and_events(
        &self,
        instance_id: &InstanceId,
    ) -> Result<ReplayResult, ReplayError> {
        match self.snapshot_store.load_latest(instance_id) {
            // Snapshot found at version N: replay events from N+1
            Ok(Some((snapshot_version, state))) => {
                let start_seq = snapshot_version.saturating_add(1);
                let events = self.collect_events(instance_id, start_seq)?;
                Ok(ReplayResult {
                    snapshot_version,
                    state,
                    events,
                })
            }
            // No snapshot: replay all events from v0
            Ok(None) => {
                let events = self.collect_events(instance_id, 0)?;
                Err(ReplayError::NoSnapshot { events })
            }
            // Snapshot corrupt: fall back to full replay
            Err(e) => {
                let fallback_events = self.collect_events(instance_id, 0)?;
                if fallback_events.is_empty() {
                    return Err(ReplayError::SnapshotLoadFailed {
                        reason: format!("{e:?}"),
                    });
                }
                Err(ReplayError::NoSnapshot { events: fallback_events })
            }
        }
    }
}
```

### Loading the Latest Snapshot

Uses reverse prefix scan (`.next_back()`) to get the highest sequence:

```rust
pub fn snapshot_load_latest(
    partition: &fjall::Keyspace,
    instance_id: &InstanceId,
) -> Result<Option<(u64, InstanceState)>, StorageError> {
    let prefix = instance_id.to_bytes().map_err(|_| StorageError::CorruptKey)?;

    partition.prefix(prefix).next_back()
        .map_or(Ok(None), |guard| {
            guard.into_inner()
                .map_err(|_| StorageError::FjallError)
                .and_then(|(key, value)| {
                    let (_, sequence) = decode_snapshot_key(&key)?;
                    let state = deserialize_snapshot_value(&value)?;
                    Ok(Some((sequence, state.state)))
                })
        })
}
```

### Corruption Fallback

If a snapshot fails checksum verification or deserialization, the engine falls
back to full event replay from sequence 0. This is a deliberate safety measure:
a corrupt snapshot must never prevent state recovery.

### Snapshot Compaction

Old snapshots can be compacted to keep only the last N:

```rust
pub fn compact_snapshots(
    partition: &fjall::Keyspace,
    instance_id: &InstanceId,
    keep_last_n: u64,
) -> Result<u64, StorageError> {
    let prefix = instance_id.to_bytes().map_err(|_| StorageError::CorruptKey)?;
    let mut snapshots: Vec<(u64, Vec<u8>)> = Vec::new();
    for item in partition.prefix(prefix) {
        let (key, value) = item.into_inner().map_err(|_| StorageError::FjallError)?;
        let (_, seq) = decode_snapshot_key(&key).map_err(|_| StorageError::InvalidKey)?;
        snapshots.push((seq, value.to_vec()));
    }
    if snapshots.len() <= usize::try_from(keep_last_n).unwrap_or(usize::MAX) {
        return Ok(0);
    }
    snapshots.sort_by_key(|b| std::cmp::Reverse(b.0));
    let to_delete = &snapshots[usize::try_from(keep_last_n).unwrap_or(usize::MAX)..];
    let mut deleted = 0u64;
    for (seq, _) in to_delete {
        let key = encode_snapshot_key(instance_id, *seq)?;
        if partition.remove(key).is_ok() { deleted += 1; }
    }
    Ok(deleted)
}
```

---

## 5. CQRS Read Models

### Projections

Veloxide materializes read models into separate partitions. The `instances`
partition is a projection of the event stream, maintaining a secondary index
keyed by `(status, created_at, instance_id)`:

```rust
/// Instance index key: [status_byte(1)][created_at_u64_be(8)][instance_id(16)]
pub fn encode_instance_index_key(
    status: InstanceStatus,
    created_at: TimestampMs,
    instance_id: &InstanceId,
) -> Result<[u8; 25], StorageError> {
    let id_bytes = instance_id.to_bytes().map_err(|_| StorageError::CorruptKey)?;
    let mut key = [0u8; 25];
    key[0] = status.to_byte();
    key[1..9].copy_from_slice(&created_at.as_u64().to_be_bytes());
    key[9..25].copy_from_slice(&id_bytes);
    Ok(key)
}
```

### Projection Schema Versioning

Veloxide uses `ProjectionCompatibilityWindow` to track which schema versions
are currently supported, enabling safe rollout of new projection formats:

```rust
pub struct ProjectionCompatibilityWindow {
    min_version: u16,
    max_version: u16,
}

/// Check if a projection payload is compatible with the current engine.
pub fn is_projection_compatible(
    window: &ProjectionCompatibilityWindow,
    schema_version: u16,
) -> bool {
    schema_version >= window.min_version && schema_version <= window.max_version
}
```

### Subscriber Cursors

The `dedupe` partition serves as an exactly-once guarantee mechanism.
Idempotency keys are checked before processing to prevent duplicate writes:

```rust
// Dedupe key: length-prefixed idempotency key
pub fn encode_dedupe_key(idempotency_key: &str) -> Vec<u8> {
    encode_length_prefixed(idempotency_key.as_bytes())
}
```

### Eventual Consistency

Projections are updated asynchronously through the `Appender` write path.
Events are classified into three tiers:

1. **CriticalControlPlane** -- event writes, always processed, never dropped
2. **OperatorProjection** -- projection updates, may be delayed under pressure
3. **BulkBlob** -- blob writes, lowest priority

### Rebuild from Scratch

If a projection becomes corrupted or outdated, it can be rebuilt by replaying
all events and re-materializing:

```rust
fn rebuild_instances_projection(
    db: &fjall::Database,
) -> Result<(), StorageError> {
    let events = db.keyspace("events", fjall::KeyspaceCreateOptions::default)?;
    let instances = db.keyspace("instances", fjall::KeyspaceCreateOptions::default)?;

    // Clear the projection
    // ... (iterate and delete all keys in instances partition)

    // Replay all events and rebuild
    for item in events.iter() {
        let (_key, value) = item.into_inner().map_err(|_| StorageError::FjallError)?;
        let envelope = EventEnvelope::from_bytes(&value)
            .map_err(|_| StorageError::CorruptEventPayload)?;
        // Apply event to rebuild the projection entry
        // instance_index_upsert(...)
    }
    Ok(())
}
```

---

## 6. Optimistic Concurrency Control (OCC)

### Expected Version Check

Veloxide uses OCC to prevent lost updates when multiple actors attempt to
append events to the same instance stream concurrently.

The `InMemoryEventStore` demonstrates the OCC contract:

```rust
async fn append(
    &self,
    instance_id: &InstanceId,
    events: Vec<EventEnvelope>,
) -> Result<u64, EventStoreError> {
    // 1. Read current sequence (expected version)
    let expected_sequence = self.sequences.read().unwrap()
        .get(instance_id).copied().unwrap_or(0);

    // 2. Validate first event follows current position
    let first_sequence = events.first().unwrap().sequence;
    if first_sequence != expected_sequence + 1 {
        return Err(EventStoreError::OccConflict {
            instance_id: instance_id.to_string(),
            expected_sequence: expected_sequence + 1,
            actual_sequence: first_sequence,
        });
    }

    // 3. Validate internal ordering of the batch
    for window in events.windows(2) {
        if let [a, b] = window {
            if b.sequence != a.sequence + 1 {
                return Err(EventStoreError::InvalidArgument {
                    reason: format!(
                        "events not sequentially ordered: {} followed by {}",
                        a.sequence, b.sequence
                    ),
                });
            }
        }
    }

    // 4. Commit: append and update sequence
    let final_sequence = events.last().unwrap().sequence;
    // ... write events, update sequence tracker ...

    Ok(final_sequence)
}
```

### OCC Error Types

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EventStoreError {
    #[error("OCC conflict for {instance_id}: expected {expected_sequence}, found {actual_sequence}")]
    OccConflict {
        instance_id: String,
        expected_sequence: u64,
        actual_sequence: u64,
    },
    #[error("storage error: {reason}")]
    Storage { reason: String },
    #[error("invalid argument: {reason}")]
    InvalidArgument { reason: String },
}
```

The storage layer also has its own OCC variant:

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StorageError {
    #[error("optimistic concurrency conflict: expected {expected_version}, found {actual_version}")]
    OptimisticConcurrency {
        expected_version: u64,
        actual_version: u64,
    },
    // ...
}
```

### OCC Resolution

On OCC conflict, the caller must:
1. Re-read the current stream state
2. Re-derive the events based on new state
3. Re-attempt the append with corrected sequence numbers

Veloxide's actor model (`ractor`) guarantees at-most-one active actor per
instance, so OCC conflicts should be rare in practice. They serve as a safety
net against race conditions during actor handoff or split-brain scenarios.

---

## 7. Key Encoding for Event Streams

### Big-Endian Sequence Numbers

All numeric components in Fjall keys use fixed-width big-endian encoding.
This ensures correct lexicographic sort order -- sequence 1, 2, ..., 10
sort correctly rather than 1, 10, 2, ...

```rust
pub const fn encode_u64_be(value: u64) -> [u8; 8] {
    value.to_be_bytes()
}

pub fn decode_u64_be(bytes: &[u8]) -> Result<u64, KeyEncodingError> {
    let arr: [u8; 8] = bytes.try_into()
        .map_err(|_| KeyEncodingError::InvalidLength { expected: 8, actual: bytes.len() })?;
    Ok(u64::from_be_bytes(arr))
}
```

### Length-Prefixed Variable Fields

Veloxide uses length-prefixed encoding for variable-length fields (step IDs,
idempotency keys) to avoid delimiter ambiguity:

```rust
pub fn encode_length_prefixed(value: &[u8]) -> Vec<u8> {
    let len = u16::try_from(value.len()).unwrap_or(u16::MAX);
    let mut result = Vec::with_capacity(2 + value.len());
    result.extend_from_slice(&len.to_be_bytes());
    result.extend_from_slice(value);
    result
}

pub fn decode_length_prefixed(bytes: &[u8]) -> Result<(&[u8], &[u8]), KeyEncodingError> {
    let len = decode_u16_be(&bytes[..2])? as usize;
    Ok((&bytes[2..2 + len], &bytes[2 + len..]))
}
```

### Composite Keys for Veloxide Partitions

**Event keys** (24 bytes fixed):
```
[InstanceId 16B | SequenceNumber 8B]
```

```rust
pub fn encode_event_key(
    instance_id: &InstanceId,
    sequence: SequenceNumber,
) -> Vec<u8> {
    let iid_bytes = instance_id.to_bytes().unwrap_or([0u8; 16]);
    let seq_bytes = encode_sequence_number(sequence);
    let mut key = Vec::with_capacity(24);
    key.extend_from_slice(&iid_bytes);
    key.extend_from_slice(&seq_bytes);
    key
}
```

**Timer keys** (variable):
```
[timestamp_u64_be(8) | iid_len_u16_be(2) | instance_id_bytes(16)]
```

**Instance index keys** (25 bytes):
```
[status_byte(1) | created_at_u64_be(8) | instance_id(16)]
```

**Effect keys** (25 bytes):
```
[instance_id(16) | sequence_u64_be(8) | 0xFF marker(1)]
```

**Lease keys** (variable):
```
[instance_id(16) | step_id_len_u16_be(2) | step_id_bytes]
```

### Prefix Scan Helper

To scan all events for an instance, use the 16-byte prefix:

```rust
pub fn get_event_key_prefix(instance_id: &InstanceId) -> Vec<u8> {
    instance_id.to_bytes().unwrap_or([0u8; 16]).to_vec()
}
```

---

## 8. Event Schema Evolution

### Versioned Types

The `EventEnvelope` includes a `schema_version` field:

```rust
pub struct EventEnvelope {
    pub schema_version: u16,  // version of the envelope format
    pub instance_id: String,
    pub sequence: u64,
    pub timestamp_ms: u64,
    pub payload: serde_json::Value,  // flexible JSON for forward compat
    pub metadata: EventMetadata,
}
```

The `CURRENT_SNAPSHOT_VERSION` tracks the latest snapshot schema:

```rust
pub const CURRENT_SNAPSHOT_VERSION: u16 = 1;
pub const MIN_SNAPSHOT_VERSION: u16 = 1;
```

### Snapshot Compatibility Checking

When loading snapshots, Veloxide validates the schema version against a
compatibility window:

```rust
pub fn snapshot_load_latest_with_compat(
    partition: &fjall::Keyspace,
    instance_id: &InstanceId,
    min_version: u16,
    engine_version: u16,
) -> Result<Option<CompatSnapshotLoad>, StorageError> {
    // ... iterate snapshots ...
    let load_result = if ds.schema_version == 0 {
        // Legacy format, always discard
        CompatSnapshotLoad::Discarded {
            sequence,
            reason: SnapshotDiscardReason::VersionZero,
        }
    } else if ds.schema_version < min_version {
        CompatSnapshotLoad::Discarded {
            sequence,
            reason: SnapshotDiscardReason::VersionTooOld {
                snapshot_version: ds.schema_version,
                min_version,
            },
        }
    } else if ds.schema_version > engine_version {
        CompatSnapshotLoad::Discarded {
            sequence,
            reason: SnapshotDiscardReason::VersionTooNew {
                snapshot_version: ds.schema_version,
                engine_version,
            },
        }
    } else {
        CompatSnapshotLoad::Loaded { sequence, state: ds.state }
    };
    // ...
}
```

### Upcasters

When an event with an older `schema_version` is encountered during replay,
an upcaster transforms it to the current format. Because `payload` is
`serde_json::Value`, this can be done without changing the on-disk format:

```rust
fn upcast_event(mut envelope: EventEnvelope) -> EventEnvelope {
    match envelope.schema_version {
        1 => {
            // Add new field with default value
            if let Some(obj) = envelope.payload.as_object_mut() {
                obj.entry("new_field")
                    .or_insert(serde_json::json!("default_value"));
            }
            envelope.schema_version = 2;
            envelope
        }
        v if v >= CURRENT_EVENT_VERSION => envelope,
        _ => panic!("unsupported event version"),
    }
}
```

---

## 9. Veloxide-Specific Patterns

### Lineage-Aware Queries

Veloxide supports three query scopes for event replay, defined as
`LineageQuery`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineageQuery<'a> {
    /// Events for a single instance only
    InstanceId(&'a InstanceId),

    /// Events for all instances in a lineage (cross-epoch)
    LineageWide { lineage_id: &'a str },

    /// Events for a specific epoch within a lineage
    EpochSpecific { lineage_id: &'a str, epoch: Epoch },
}
```

### Lineage Key Encoding

Lineage-wide and epoch-specific queries use 0xFF-delimited encoding:

```rust
pub const LINEAGE_ID_NULL_BYTE: u8 = 0xFF;

/// Lineage prefix: [0xFF][lineage_id_bytes][0xFF]
pub fn lineage_prefix_generator(lineage_id: &str) -> Result<Vec<u8>, StorageError> {
    let mut prefix = Vec::with_capacity(1 + lineage_id.len() + 1);
    prefix.push(LINEAGE_ID_NULL_BYTE);     // 0xFF sentinel
    prefix.extend_from_slice(lineage_id.as_bytes());
    prefix.push(LINEAGE_ID_NULL_BYTE);     // 0xFF terminator
    Ok(prefix)
}

/// Epoch prefix: [0xFF][lineage_id_bytes][0xFF][epoch_u64_be(8)]
pub fn epoch_prefix_generator(
    lineage_id: &str,
    epoch: Epoch,
) -> Result<Vec<u8>, StorageError> {
    let mut prefix = lineage_prefix_generator(lineage_id)?;
    prefix.extend_from_slice(&epoch.get().to_be_bytes());
    Ok(prefix)
}
```

### LineageReplayIterator

The `LineageReplayIterator` handles cross-epoch replay. When scanning a
lineage-wide prefix, it resets the sequence validator when crossing epoch
boundaries (since each epoch has its own sequence numbering starting from 1):

```rust
pub struct LineageReplayIterator {
    inner: Option<Box<dyn DoubleEndedIterator<Item = fjall::Guard>>>,
    state: IteratorState,
    prefix_len: usize,
    epoch_len: usize,        // 8 for LineageWide, 0 for InstanceId/EpochSpecific
    last_epoch: Vec<u8>,
    init_error: Option<StorageError>,
}

impl Iterator for LineageReplayIterator {
    type Item = Result<EventEnvelope, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        // ... for each key-value pair from the range scan ...
        // Detect epoch boundary crossing:
        if self.epoch_len > 0 && self.state.started {
            let epoch_bytes = &k_bytes[self.prefix_len..self.prefix_len + self.epoch_len];
            if epoch_bytes != self.last_epoch.as_slice() {
                // New epoch: reset sequence validator
                self.state = IteratorState::new();
                self.last_epoch = epoch_bytes.to_vec();
            }
        }
        // Validate sequence within the epoch
        match self.state.advance(found_seq, envelope) {
            Some(Err(e)) => { /* gap detected */ }
            Some(Ok(env)) => return Some(Ok(env)),
            None => continue, // epoch boundary, continue scanning
        }
    }
}
```

### Continue-as-New (WorkflowLineage)

Workflows can "continue as new" -- rolling over to a fresh instance while
preserving the lineage. The `WorkflowLineage` type tracks this:

```rust
pub struct WorkflowLineage {
    lineage_id: String,       // stable across epochs
    epoch: Epoch,             // current epoch number
    parent_epoch: Option<Epoch>, // previous epoch (set after first rollover)
}

impl WorkflowLineage {
    /// Roll over to a new epoch via continue-as-new.
    pub fn continue_as_new(&self) -> Result<Self, LineageError> {
        let next_epoch = self.epoch.get().checked_add(1)
            .ok_or(LineageError::EpochOverflow)?;
        Self::with_parent(
            self.lineage_id.clone(),
            Epoch::new(next_epoch),
            Some(self.epoch),
        )
    }
}
```

The `lineage_store` module persists `LineageRecord` for routing signals to
the active epoch:

```rust
pub struct LineageRecord {
    pub lineage_id: String,
    pub active_epoch: Epoch,
    pub active_instance_id: InstanceId,
    pub previous_instance_id: Option<InstanceId>,
}
```

### FjallEventStore (TODO)

The production Fjall-backed event store is currently marked TODO in
`StorageEngine` (removed during fjall 3 migration):

```rust
pub struct StorageEngine {
    db: fjall::Database,
    pub dedupe_store: Arc<FjallDedupeStore>,
    pub effect_journal: Arc<FjallEffectJournal>,
    pub lease_store: Arc<FjallLeaseStore>,
    // TODO: event_store module removed during fjall 3 migration - needs reimplementation
    // pub event_store: Arc<FjallEventStore>,
}
```

The `InMemoryEventStore` serves as the reference implementation of the
`EventStore` trait until the Fjall version is reimplemented.

### BudgetQueues Write Path

The `BudgetQueues` system provides QoS-aware write queuing with three
priority tiers:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WriteClass {
    CriticalControlPlane,   // tier 1: events, never dropped
    OperatorProjection,     // tier 2: projections
    BulkBlob,               // tier 3: blobs
}

/// Entry types for the write path
pub enum AppendEntry {
    ControlPlane(ControlPlaneWrite),
    Projection(ProjectionWrite),
    Blob(BlobWrite),
}
```

Priority dequeue always services critical writes first:

```rust
pub fn dequeue_prioritized(&self) -> Option<(WriteClass, T)> {
    if let Some(item) = self.dequeue(WriteClass::CriticalControlPlane) {
        return Some((WriteClass::CriticalControlPlane, item));
    }
    if let Some(item) = self.dequeue(WriteClass::OperatorProjection) {
        return Some((WriteClass::OperatorProjection, item));
    }
    if let Some(item) = self.dequeue(WriteClass::BulkBlob) {
        return Some((WriteClass::BulkBlob, item));
    }
    None
}
```

---

## 10. Cross-Partition Atomicity

### OwnedWriteBatch

Fjall provides `OwnedWriteBatch` for atomic writes across multiple partitions.
A single batch commits via a single WAL fsync, ensuring atomicity:

```rust
fn atomic_status_transition(
    db: &fjall::Database,
    partition: &fjall::Keyspace,
    old_key: &[u8; 25],
    new_key: &[u8; 25],
) -> Result<(), StorageError> {
    let mut batch = db.batch();
    batch.remove(partition, *old_key);
    batch.insert(partition, *new_key, &[] as &[u8]);
    batch.commit().map_err(|_| StorageError::Storage)
}
```

### Atomic Snapshot + Event Writes

The `AtomicSnapshotWriter` writes snapshots into a shared batch, allowing
snapshot writes to be co-committed with event writes:

```rust
pub struct AtomicSnapshotWriter<'a> {
    db: &'a fjall::Database,
    snapshot_partition: Keyspace,
}

impl<'a> AtomicSnapshotWriter<'a> {
    /// Add a snapshot write to an existing batch (no commit yet).
    pub fn write_snapshot(
        &self,
        batch: &mut fjall::OwnedWriteBatch,
        instance_id: InstanceId,
        sequence: u64,
        state: &InstanceState,
    ) -> Result<(), StorageError> {
        let key = encode_snapshot_key(&instance_id, sequence)?;
        let state_json = serde_json::to_vec(state)
            .map_err(|_| StorageError::SerializationFailed)?;
        let checksum = crc32fast::hash(&state_json);
        let header = SnapshotHeader::new(instance_id, sequence, checksum);
        let header_bytes = serde_json::to_vec(&header)
            .map_err(|_| StorageError::SerializationFailed)?;
        let mut value = header_bytes;
        value.push(b'|');
        value.extend_from_slice(&state_json);
        batch.insert(&self.snapshot_partition, key, &value);
        Ok(())
    }

    /// Convenience: create a dedicated batch, write snapshot, commit atomically.
    pub fn write_snapshot_atomic(
        &self,
        instance_id: InstanceId,
        sequence: u64,
        state: &InstanceState,
    ) -> Result<(), StorageError> {
        let mut batch = self.db.batch();
        self.write_snapshot(&mut batch, instance_id, sequence, state)?;
        batch.commit().map_err(|_| StorageError::BatchCommitFailed)
    }
}
```

### Group Commit Through DbWriterActor Pattern

Veloxide's architecture (per CLAUDE.md) mandates that **actors NEVER write to
fjall directly**. All writes go through a centralized `DbWriterActor` (a
`ractor` actor) that:

1. Receives write commands from workflow actors via messages
2. Batches multiple writes together
3. Commits the batch as a single WAL fsync
4. Prevents SSD lock contention from concurrent writers

```
WorkflowActor --> DbWriterActor --> Fjall WriteBatch --> WAL fsync --> Done
                    ^
                    |
ProjectionActor ----+
                    |
TimerActor ---------+
```

This pattern ensures:
- **Single writer discipline** -- only one thread writes to Fjall at a time
- **Batching efficiency** -- multiple small writes become one fsync
- **Backpressure** -- the `BudgetQueues` system throttles incoming writes
  when the writer is saturated
- **Atomicity** -- related writes across partitions commit together

### Example: Atomic Event + Index Update

To atomically append an event and update the instance index:

```rust
fn append_event_with_index(
    db: &fjall::Database,
    instance_id: &InstanceId,
    event: EventEnvelope,
    status: InstanceStatus,
    created_at: TimestampMs,
) -> Result<(), StorageError> {
    let events_partition = db.keyspace("events", || {
        fjall::KeyspaceCreateOptions::default()
    })?;
    let instances_partition = db.keyspace("instances", || {
        fjall::KeyspaceCreateOptions::default()
    })?;

    let mut batch = db.batch();

    // Write the event
    let event_key = crate::key_encoding::encode_event_key(
        instance_id,
        &SequenceNumber::try_from(event.sequence).unwrap(),
    );
    let event_value = serde_json::to_vec(&event)
        .map_err(|_| StorageError::SerializationFailed)?;
    batch.insert(&events_partition, event_key, &event_value);

    // Update the instance index
    let index_key = encode_instance_index_key(status, created_at, instance_id)?;
    batch.insert(&instances_partition, index_key, &[] as &[u8]);

    // Single WAL fsync for both writes
    batch.commit().map_err(|_| StorageError::BatchCommitFailed)
}
```
