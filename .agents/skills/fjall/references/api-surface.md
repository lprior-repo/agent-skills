# Fjall V3 (3.1.4) Complete API Surface Reference

Crate: `fjall` 3.1.4 | MSRV: 1.90.0 | Edition: 2021
Underlying LSM-tree: `lsm-tree` 3.1.4 | Backed by `byteview` for zero-copy slices

---

## Table of Contents

1. [Database Construction](#1-database-construction)
2. [Keyspace Operations](#2-keyspace-operations)
3. [OwnedWriteBatch](#3-ownedwritebatch)
4. [Iteration](#4-iteration)
5. [Guard Iterator API](#5-guard-iterator-api)
6. [Snapshots](#6-snapshots)
7. [Slice Type](#7-slice-type)
8. [Transactions](#8-transactions)
9. [Compaction Filters](#9-compaction-filters)
10. [KV Separation](#10-kv-separation)
11. [Compression](#11-compression)
12. [Bloom Filters](#12-bloom-filters)
13. [Bulk Loading](#13-bulk-loading)
14. [Durability](#14-durability)
15. [Key Types and Errors](#15-key-types-and-errors)

---

## 1. Database Construction

Fjall provides three database variants, all opened via a typed builder pattern.

### Database (Non-Transactional)

```rust
use fjall::{Database, KeyspaceCreateOptions};

let db = Database::builder("/path/to/db")
    .cache_size(64 * 1024 * 1024)          // Block cache: default 32 MiB
    .journal_compression(CompressionType::Lz4)
    .manual_journal_persist(false)          // Default: false (auto-flush)
    .worker_threads(4)                      // Default: min(CPU cores, 4)
    .max_cached_files(Some(900))            // Default: 900 (Linux), 400 (Win), 150 (Mac)
    .max_journaling_size(512 * 1024 * 1024) // Default: 512 MiB. Min: 64 MiB.
    .temporary(false)                       // Delete path on drop
    .with_compaction_filter_factories(Arc::new(|keyspace_name| {
        // Return Option<Arc<dyn Factory>> based on keyspace name
        None
    }))
    .open()?;
```

**Type signature:**
```rust
impl Database {
    pub fn builder(path: impl AsRef<Path>) -> DatabaseBuilder<Database>;
    pub fn open(config: Config) -> Result<Database>;
    pub fn create_or_recover(config: Config) -> Result<Database>;  // hidden, no bg threads
}

#[derive(Clone)]
pub struct Database(Arc<DatabaseInner>);  // Clone is cheap (Arc)
```

**Builder methods (all return `Self`, all `#[must_use]`):**

| Method | Type | Default | Notes |
|--------|------|---------|-------|
| `cache_size` | `u64` bytes | 32 MiB | ~20-25% of available memory recommended |
| `journal_compression` | `CompressionType` | `Lz4` (with feature) | Compression for journal values |
| `manual_journal_persist` | `bool` | `false` | If true, handle persist manually |
| `worker_threads` | `usize` | min(CPU, 4) | Panics if 0 |
| `max_cached_files` | `Option<usize>` | platform-dependent | Panics if <10 or None |
| `max_journaling_size` | `u64` bytes | 512 MiB | Panics if <64 MiB |
| `temporary` | `bool` | `false` | Deletes path on drop |
| `with_compaction_filter_factories` | `CompactionFilterAssigner` | None | `Arc<dyn Fn(&str) -> Option<Arc<dyn Factory>>>` |

**Database methods:**

```rust
impl Database {
    // Keyspace management
    pub fn keyspace(&self, name: &str, create_options: impl FnOnce() -> KeyspaceCreateOptions) -> Result<Keyspace>;
    pub fn delete_keyspace(&self, handle: Keyspace) -> Result<()>;
    pub fn keyspace_exists(&self, name: &str) -> bool;
    pub fn keyspace_count(&self) -> usize;
    pub fn list_keyspace_names(&self) -> Vec<KeyspaceKey>;  // KeyspaceKey = byteview::StrView

    // Batch writes
    pub fn batch(&self) -> OwnedWriteBatch;

    // Snapshot
    pub fn snapshot(&self) -> Snapshot;

    // Durability
    pub fn persist(&self, mode: PersistMode) -> Result<()>;

    // Diagnostics
    pub fn disk_space(&self) -> Result<u64>;
    pub fn journal_count(&self) -> usize;
    pub fn journal_disk_space(&self) -> Result<u64>;       // hidden
    pub fn write_buffer_size(&self) -> u64;                 // hidden
    pub fn outstanding_flushes(&self) -> usize;             // hidden
    pub fn active_compactions(&self) -> usize;              // hidden
    pub fn compactions_completed(&self) -> usize;           // hidden
    pub fn time_compacting(&self) -> Duration;              // hidden
}
```

**Keyspace name constraints:** 1-255 characters, non-empty. Panics on invalid name.

---

## 2. Keyspace Operations

Each keyspace is a physically independent LSM-tree.

```rust
let tree = db.keyspace("events", KeyspaceCreateOptions::default())?;
```

### Keyspace Handle

```rust
#[derive(Clone)]
pub struct Keyspace(Arc<KeyspaceInner>);  // Cheap to clone
```

### CRUD Operations

```rust
// Insert (overwrites if exists)
pub fn insert<K: Into<UserKey>, V: Into<UserValue>>(&self, key: K, value: V) -> Result<()>;

// Get
pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<UserValue>>;

// Get size only (avoids loading value blob in KV-separated trees)
pub fn size_of<K: AsRef<[u8]>>(&self, key: K) -> Result<Option<u32>>;

// Remove (tombstone)
pub fn remove<K: Into<UserKey>>(&self, key: K) -> Result<()>;

// Remove with weak tombstone (experimental, see docs)
pub fn remove_weak<K: Into<UserKey>>(&self, key: K) -> Result<()>;  // hidden

// Check existence
pub fn contains_key<K: AsRef<[u8]>>(&self, key: K) -> Result<bool>;

// Clear entire keyspace in O(1)
pub fn clear(&self) -> Result<()>;
```

### Query Operations

```rust
// Exact count (O(n), scans entire keyspace)
pub fn len(&self) -> Result<usize>;

// O(log N) emptiness check
pub fn is_empty(&self) -> Result<bool>;

// O(1) approximate count (inaccurate after deletes/updates)
pub fn approximate_len(&self) -> usize;

// First/last key-value pairs
pub fn first_key_value(&self) -> Option<Guard>;
pub fn last_key_value(&self) -> Option<Guard>;
```

### Diagnostics

```rust
pub fn name(&self) -> &KeyspaceKey;       // byteview::StrView
pub fn path(&self) -> &Path;
pub fn disk_space(&self) -> u64;
pub fn is_kv_separated(&self) -> bool;
pub fn fragmented_blob_bytes(&self) -> u64;
```

### KeyspaceCreateOptions

```rust
let opts = KeyspaceCreateOptions::default()
    .max_memtable_size(64 * 1024 * 1024)              // Default: 64 MiB
    .with_kv_separation(Some(KvSeparationOptions::default()))
    .data_block_size_policy(BlockSizePolicy::all(4_096))
    .data_block_restart_interval_policy(RestartIntervalPolicy::new([10, 16]))
    .data_block_compression_policy(CompressionPolicy::new([CompressionType::None, CompressionType::Lz4]))
    .index_block_compression_policy(CompressionPolicy::disabled())
    .filter_policy(FilterPolicy::new([
        FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)),
    ]))
    .expect_point_read_hits(false)
    .filter_block_pinning_policy(PinningPolicy::new([true, false]))
    .index_block_pinning_policy(PinningPolicy::new([true, true, false]))
    .filter_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]))
    .index_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]))
    .compaction_strategy(Arc::new(fjall::compaction::Leveled::default()))
    .manual_journal_persist(false);
```

**Key defaults for `KeyspaceCreateOptions::default()`:**

| Setting | Default |
|---------|---------|
| `max_memtable_size` | 64 MiB |
| `data_block_size_policy` | 4 KiB (all levels) |
| `data_block_restart_interval` | [10, 16] |
| `data_block_compression` | [None, None, Lz4] (with lz4 feature) |
| `index_block_compression` | None (all levels) |
| `filter_policy` | [Bloom(FPR 0.0001), Bloom(BPK 10.0)] |
| `expect_point_read_hits` | false |
| `compaction_strategy` | Leveled (L0 threshold=4, target_size=64MiB) |
| `kv_separation_opts` | None |
| `data_block_hash_ratio` | 0.0 (disabled) |

---

## 3. OwnedWriteBatch

Atomic cross-keyspace writes. All items in a batch are committed to the journal as a single unit.

```rust
let mut batch = db.batch();

batch.insert(&tree1, "key1", "value1");
batch.insert(&tree2, "key2", "value2");
batch.remove(&tree1, "old_key");

batch.commit()?;
```

### Type and Construction

```rust
pub struct OwnedWriteBatch {
    data: Vec<Item>,          // internal
    db: Database,
    durability: Option<PersistMode>,
}

// Create via Database
impl Database {
    pub fn batch(&self) -> OwnedWriteBatch;
}

// Pre-allocated capacity (item count, not byte size)
OwnedWriteBatch::with_capacity(db, 1000);
```

### Methods

```rust
impl OwnedWriteBatch {
    pub fn insert<K: Into<UserKey>, V: Into<UserValue>>(&mut self, keyspace: &Keyspace, key: K, value: V);
    pub fn remove<K: Into<UserKey>>(&mut self, keyspace: &Keyspace, key: K);
    pub fn remove_weak<K: Into<UserKey>>(&mut self, keyspace: &Keyspace, key: K);  // hidden/experimental
    pub fn durability(self, mode: Option<PersistMode>) -> Self;
    pub fn commit(self) -> Result<()>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

**Constraints:**
- Keys: up to 65536 bytes (`u16::MAX`)
- Values: up to 2^32 bytes (`u32::MAX`)
- Empty batch commits are no-ops
- Batch acquires journal writer mutex during commit

**Performance:** All items share a single journal write, a single seqno, and a single persist. This is significantly faster than individual inserts for bulk data.

---

## 4. Iteration

### Iterator Types

```rust
pub struct Iter {
    iter: Box<dyn DoubleEndedIterator<Item = lsm_tree::IterGuardImpl> + Send + 'static>,
    nonce: SnapshotNonce,  // Keeps snapshot alive for GC safety
}
```

### Creating Iterators

```rust
impl Keyspace {
    // Full scan
    pub fn iter(&self) -> Iter;

    // Range scan
    pub fn range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Iter;

    // Prefix scan
    pub fn prefix<K: AsRef<[u8]>>(&self, prefix: K) -> Iter;
}
```

### Iterator Traits

```rust
impl Iterator for Iter {
    type Item = Guard;
    fn next(&mut self) -> Option<Guard>;
}

impl DoubleEndedIterator for Iter {
    fn next_back(&mut self) -> Option<Guard>;
}
```

### Usage Patterns

```rust
// Forward iteration
for guard in tree.iter() {
    let key = guard.key()?;
    // ...
}

// Reverse iteration
for guard in tree.iter().rev() {
    let key = guard.key()?;
}

// Range scan
for guard in tree.range("a"..="z") {
    let (k, v) = guard.into_inner()?;
}

// Prefix scan (reverse)
for guard in tree.prefix("user:").rev() {
    let key = guard.key()?;
}

// First/last via iterator
let first = tree.iter().next();
let last = tree.iter().next_back();
```

**Performance notes:**
- Each iterator opens a snapshot nonce to prevent GC of visible data
- Iterators that scan the full keyspace are O(n) -- avoid or limit them
- Prefix scan uses range optimization under the hood
- DoubleEndedIterator allows efficient reverse scans without reversing in memory

---

## 5. Guard Iterator API (V3 Lazy Blob Loading)

Guards provide lazy value loading -- especially important for KV-separated trees where value blobs are only loaded on demand.

```rust
pub struct Guard(pub(crate) lsm_tree::IterGuardImpl);
```

### Methods

```rust
impl Guard {
    /// Returns the key. Consumes the guard.
    /// Triggers blob load if KV-separated (but only key, not value).
    pub fn key(self) -> Result<UserKey>;

    /// Returns the value. Consumes the guard.
    /// Triggers blob load if KV-separated.
    pub fn value(self) -> Result<UserValue>;

    /// Returns the value size without loading the value.
    /// O(1) -- reads metadata only.
    pub fn size(self) -> Result<u32>;

    /// Returns the full key-value pair. Consumes the guard.
    /// Always loads the value.
    pub fn into_inner(self) -> Result<KvPair>;

    /// Conditionally loads the value based on a key predicate.
    /// If predicate returns false, value may not be loaded (saves blob I/O).
    pub fn into_inner_if(
        self,
        pred: impl Fn(&UserKey) -> bool,
    ) -> Result<(UserKey, Option<UserValue>)>;
}
```

### Usage Examples

```rust
// Only load value if key matches a pattern
for guard in tree.prefix("important:") {
    let (key, maybe_value) = guard.into_inner_if(|k| k.starts_with(b"important:active:"))?;
    if let Some(value) = maybe_value {
        // Process value
    }
}

// Get size without loading blob
for guard in tree.iter() {
    let sz = guard.size()?;
    if sz < 1024 {
        // only load small values
    }
}

// Get key only (most efficient)
for guard in tree.prefix("idx:") {
    let key = guard.key()?;
    // key-only scan, no value I/O
}
```

**Performance notes:**
- `key()` and `size()` never load value blobs
- `into_inner_if()` can skip blob I/O entirely when predicate returns false
- For KV-separated trees, value access triggers a separate blob file read
- For standard trees, all data is inline so loading is always cheap

---

## 6. Snapshots (Point-in-Time Reads)

Snapshots provide MVCC-consistent views of the database at a specific sequence number.

```rust
#[derive(Clone)]
#[clippy::has_significant_drop]
pub struct Snapshot {
    nonce: SnapshotNonce,
}
```

### Creating Snapshots

```rust
// From Database
let snapshot = db.snapshot();

// From SingleWriterTxDatabase
let snapshot = tx_db.read_tx();

// From OptimisticTxDatabase
let snapshot = optimistic_db.read_tx();
```

### Snapshot Methods (via Readable trait)

```rust
pub trait Readable {
    fn get<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<Option<UserValue>>;
    fn contains_key<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<bool>;
    fn size_of<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<Option<u32>>;
    fn first_key_value(&self, keyspace: impl AsRef<Keyspace>) -> Option<Guard>;
    fn last_key_value(&self, keyspace: impl AsRef<Keyspace>) -> Option<Guard>;
    fn iter(&self, keyspace: impl AsRef<Keyspace>) -> Iter;
    fn range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, keyspace: impl AsRef<Keyspace>, range: R) -> Iter;
    fn prefix<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, prefix: K) -> Iter;
    fn is_empty(&self, keyspace: impl AsRef<Keyspace>) -> Result<bool>;
    fn len(&self, keyspace: impl AsRef<Keyspace>) -> Result<usize>;
}
```

### Usage Example

```rust
let snapshot = db.snapshot();

// Read from snapshot -- repeatable reads
let v1 = snapshot.get(&tree, "key")?;

// Writes after snapshot creation are NOT visible
tree.insert("key", "new_value")?;

let v2 = snapshot.get(&tree, "key")?;  // Returns original value, not "new_value"
assert_eq!(v1, v2);

// Snapshot iteration
for guard in snapshot.prefix(&tree, "user:") {
    let (k, v) = guard.into_inner()?;
}
```

**MVCC behavior:**
- Snapshot pins a seqno, preventing GC of versions visible to it
- `#[clippy::has_significant_drop]` -- dropping the snapshot releases the GC pin
- Keep snapshots short-lived to prevent unbounded space growth
- For serializable isolation, use transactional databases instead

---

## 7. Slice Type

`Slice` is the fundamental byte container used for both keys and values. Backed by `byteview::ByteView`, which uses `Arc<[u8]>` internally with inline optimization for small values.

```rust
pub type UserKey = Slice;
pub type UserValue = Slice;
pub type KvPair = (UserKey, UserValue);
```

### Definition

```rust
#[derive(Debug, Clone, Eq, Hash, Ord)]
pub struct Slice(pub(super) ByteView);
```

### Construction

```rust
// From various types
Slice::from(b"hello".as_slice());       // &[u8]
Slice::from(vec![1, 2, 3]);             // Vec<u8>
Slice::from("hello");                   // &str
Slice::from(String::from("hello"));     // String
Slice::from([1u8, 2, 3]);              // [u8; N]
Slice::from(Arc::<[u8]>::from([1, 2])); // Arc<[u8]>
Slice::from(path_buf);                  // PathBuf, &Path
Slice::from(Arc::<str>::from("hi"));    // Arc<str>

// From iterator
Slice::from_iter(vec![1u8, 2, 3]);

// From reader
Slice::from_reader(&mut cursor, 4)?;

// Empty
Slice::empty();

// Fuse two slices
Slice::fused(b"abc", b"def");           // "abcdef"

// Static methods
Slice::new(bytes: &[u8]);
```

### Behavior

```rust
impl std::ops::Deref for Slice {
    type Target = [u8];
    // Allows &slice[n], slice.len(), etc.
}

impl PartialEq<T> for Slice where T: AsRef<[u8]> { ... }
impl PartialOrd<T> for Slice where T: AsRef<[u8]> { ... }
impl FromIterator<u8> for Slice { ... }
impl Borrow<[u8]> for Slice { ... }
```

**Performance notes:**
- `Clone` is O(1) -- just bumps Arc refcount
- Inline storage for small values (up to ~20 bytes in ByteView)
- Zero-copy when constructed from `Vec<u8>` or `Arc<[u8]>`
- `Hash` and `Ord` based on byte content
- `Slice` derefs to `[u8]` so all slice operations work directly

---

## 8. Transactions

Fjall provides two transactional database variants with different concurrency strategies.

### SingleWriterTxDatabase (Serialized)

Single writer -- uses a `Mutex<()>` to serialize all write transactions. No conflicts possible.

```rust
use fjall::{SingleWriterTxDatabase, KeyspaceCreateOptions, Readable};

let db = SingleWriterTxDatabase::builder("/path").open()?;
let tree = db.keyspace("default", KeyspaceCreateOptions::default())?;

// Write transaction
let mut tx = db.write_tx();
tx.insert(&tree, "key", "value");
tx.remove(&tree, "old_key");

// Read-your-own-writes
let val = tx.get(&tree, "key")?;

// Commit or rollback
tx.commit()?;
// tx.rollback();  // explicit rollback

// Read-only transaction (snapshot)
let read_tx = db.read_tx();
let val = read_tx.get(&tree, "key")?;
```

**Single-writer write transaction:**
```rust
pub struct WriteTransaction<'a> {
    _guard: MutexGuard<'a, ()>,  // Holds the single-writer lock
    inner: BaseTransaction,
}

impl<'tx> WriteTransaction<'tx> {
    pub fn durability(self, mode: Option<PersistMode>) -> Self;
    pub fn insert<K, V>(&mut self, keyspace: &SingleWriterTxKeyspace, key: K, value: V);
    pub fn remove<K>(&mut self, keyspace: &SingleWriterTxKeyspace, key: K);
    pub fn remove_weak<K>(&mut self, keyspace: &SingleWriterTxKeyspace, key: K);
    pub fn take<K>(&mut self, keyspace: &SingleWriterTxKeyspace, key: K) -> Result<Option<UserValue>>;
    pub fn update_fetch<K, F>(&mut self, keyspace: &SingleWriterTxKeyspace, key: K, f: F) -> Result<Option<UserValue>>;
    pub fn fetch_update<K, F>(&mut self, keyspace: &SingleWriterTxKeyspace, key: K, f: F) -> Result<Option<UserValue>>;
    pub fn commit(self) -> Result<()>;
    pub fn rollback(self);
}
```

Implements `Readable` (get, contains_key, iter, range, prefix, first/last_key_value, size_of).

**SingleWriterTxKeyspace convenience methods** (each wraps an implicit transaction):
```rust
impl SingleWriterTxKeyspace {
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<()>;
    pub fn remove<K>(&self, key: K) -> Result<()>;
    pub fn remove_weak<K>(&self, key: K) -> Result<()>;
    pub fn take<K>(&self, key: K) -> Result<Option<UserValue>>;
    pub fn fetch_update<K, F>(&self, key: K, f: F) -> Result<Option<UserValue>>;
    pub fn update_fetch<K, F>(&self, key: K, f: F) -> Result<Option<UserValue>>;
    pub fn get<K>(&self, key: K) -> Result<Option<UserValue>>;
    pub fn contains_key<K>(&self, key: K) -> Result<bool>;
    pub fn first_key_value(&self) -> Option<Guard>;
    pub fn last_key_value(&self) -> Option<Guard>;
    pub fn size_of<K>(&self, key: K) -> Result<Option<u32>>;
    pub fn approximate_len(&self) -> usize;
    pub fn path(&self) -> PathBuf;
    pub fn inner(&self) -> &Keyspace;  // Escape transactional context
}
```

### OptimisticTxDatabase (SSI)

Serializable Snapshot Isolation using optimistic concurrency control. Multiple concurrent writers, with SSI conflict detection.

```rust
use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};

let db = OptimisticTxDatabase::builder("/path").open()?;
let tree = db.keyspace("default", KeyspaceCreateOptions::default())?;

// Write transaction -- may conflict
let mut tx = db.write_tx()?;  // Note: returns Result (can fail)
tx.insert(&tree.inner(), "key", "value");

// Read-your-own-writes
let val = tx.get(&tree.inner(), "key")?;

// Commit may detect conflicts
match tx.commit()? {
    Ok(()) => { /* success */ }
    Err(Conflict) => { /* retry */ }
}
```

**Optimistic write transaction:**
```rust
pub struct WriteTransaction {
    inner: BaseTransaction,
    cm: ConflictManager,
    oracle: Arc<Oracle>,
}

impl WriteTransaction {
    pub fn durability(self, mode: Option<PersistMode>) -> Self;
    pub fn insert<K, V>(&mut self, keyspace: impl AsRef<Keyspace>, key: K, value: V);
    pub fn remove<K>(&mut self, keyspace: impl AsRef<Keyspace>, key: K);
    pub fn remove_weak<K>(&mut self, keyspace: impl AsRef<Keyspace>, key: K);
    pub fn take<K>(&mut self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<Option<UserValue>>;
    pub fn update_fetch<K, F>(&mut self, keyspace: impl AsRef<Keyspace>, key: K, f: F) -> Result<Option<UserValue>>;
    pub fn fetch_update<K, F>(&mut self, keyspace: impl AsRef<Keyspace>, key: K, f: F) -> Result<Option<UserValue>>;
    pub fn commit(self) -> Result<Result<(), Conflict>>;
    pub fn rollback(self);
}
```

Implements `Readable` -- but note `iter`, `range`, and `prefix` track read ranges for conflict detection.

**Conflict type:**
```rust
#[derive(Debug)]
pub struct Conflict;
impl std::error::Error for Conflict {}
impl fmt::Display for Conflict { ... }
```

**OptimisticTxKeyspace convenience methods** (auto-retry on conflict for fetch_update/update_fetch):
```rust
impl OptimisticTxKeyspace {
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<()>;       // Single tx, blind write
    pub fn remove<K>(&self, key: K) -> Result<()>;                     // Single tx, blind remove
    pub fn take<K>(&self, key: K) -> Result<Option<UserValue>>;        // Auto-retry loop
    pub fn fetch_update<K, F>(&self, key: K, f: F) -> Result<Option<UserValue>>;  // Auto-retry loop, FnMut
    pub fn update_fetch<K, F>(&self, key: K, f: F) -> Result<Option<UserValue>>;  // Auto-retry loop, FnMut
    pub fn get<K>(&self, key: K) -> Result<Option<UserValue>>;
    pub fn contains_key<K>(&self, key: K) -> Result<bool>;
    pub fn first_key_value(&self) -> Option<Guard>;
    pub fn last_key_value(&self) -> Option<Guard>;
    pub fn size_of<K>(&self, key: K) -> Result<Option<u32>>;
    pub fn approximate_len(&self) -> usize;
    pub fn path(&self) -> PathBuf;
    pub fn inner(&self) -> &Keyspace;
}
```

### Readable Trait (shared by Snapshot and all WriteTransactions)

```rust
pub trait Readable {
    fn get<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<Option<UserValue>>;
    fn contains_key<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<bool>;
    fn size_of<K: AsRef<[u8]>>(&self, keyspace: impl AsRef<Keyspace>, key: K) -> Result<Option<u32>>;
    fn first_key_value(&self, keyspace: impl AsRef<Keyspace>) -> Option<Guard>;
    fn last_key_value(&self, keyspace: impl AsRef<Keyspace>) -> Option<Guard>;
    fn iter(&self, keyspace: impl AsRef<Keyspace>) -> Iter;
    fn range<K, R>(&self, keyspace: impl AsRef<Keyspace>, range: R) -> Iter;
    fn prefix<K>(&self, keyspace: impl AsRef<Keyspace>, prefix: K) -> Iter;
    fn is_empty(&self, keyspace: impl AsRef<Keyspace>) -> Result<bool>;
    fn len(&self, keyspace: impl AsRef<Keyspace>) -> Result<usize>;
}
```

### Transaction Internals

Write transactions use an in-memory `Memtable` per keyspace for RYOW (read-your-own-writes). On commit, the memtable contents are flushed into an `OwnedWriteBatch` and committed atomically. Transaction seqnos start at `0x8000_0000_0000_0000` to ensure they sort newer than any persisted data.

---

## 9. Compaction Filters (V3.1)

Compaction filters run custom logic during background compaction, enabling patterns like TTL, custom cleanup, and value transformation.

> **Note**: `CompactionFilter`, `Factory`, `Verdict`, `Context`, and `ItemAccessor` are defined in the `lsm_tree` crate and re-exported through `fjall`. Import them via `fjall::compaction::filter::{...}` or `lsm_tree::compaction::filter::{...}`. The `filter_item` return type is `lsm_tree::Result<Verdict>` (not `fjall::Result`).

### Verdict

```rust
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum Verdict {
    #[default]
    Keep,
    Remove,
    RemoveWeak,
    ReplaceValue(UserValue),
    Destroy,  // No tombstone left behind; only safe for single-write keys
}
```

### CompactionFilter Trait

```rust
pub trait CompactionFilter: Send {
    fn filter_item(&mut self, item: ItemAccessor<'_>, ctx: &Context) -> lsm_tree::Result<Verdict>;
    fn finish(self: Box<Self>) {}
}
```

### Context

```rust
#[non_exhaustive]
#[derive(Debug)]
pub struct Context {
    pub is_last_level: bool,
}
```

### Factory Trait

```rust
pub trait Factory: Send + Sync + RefUnwindSafe {
    fn name(&self) -> &str;
    fn make_filter(&self, ctx: &Context) -> Box<dyn CompactionFilter>;
}
```

### ItemAccessor

```rust
pub struct ItemAccessor<'a> { ... }

impl<'a> ItemAccessor<'a> {
    pub fn key(&self) -> &'a UserKey;
    pub fn value(&self) -> crate::Result<UserValue>;
    pub fn is_indirection(&self) -> bool;
}
```

### Registration

Compaction filters are registered at the database level via a factory assigner:

```rust
use std::sync::Arc;
use fjall::{Database, KeyspaceCreateOptions};
use fjall::compaction::filter::{Factory, CompactionFilter, Context, Verdict, ItemAccessor};

struct TtlFilter;

impl CompactionFilter for TtlFilter {
    fn filter_item(&mut self, item: ItemAccessor<'_>, _ctx: &Context) -> lsm_tree::Result<Verdict> {
        // Inspect key, value, decide verdict
        Ok(Verdict::Keep)
    }
}

struct TtlFactory;
impl Factory for TtlFactory {
    fn name(&self) -> &str { "ttl_filter" }
    fn make_filter(&self, _ctx: &Context) -> Box<dyn CompactionFilter> {
        Box::new(TtlFilter)
    }
}

let db = Database::builder("/path")
    .with_compaction_filter_factories(Arc::new(|keyspace_name| {
        match keyspace_name {
            "events" => Some(Arc::new(TtlFactory) as Arc<dyn Factory>),
            _ => None,
        }
    }))
    .open()?;
```

**Performance notes:**
- Filters run during background compaction, not on the read/write path
- `filter_item` must NOT panic
- Returning an error aborts the compaction
- `value()` on ItemAccessor triggers blob I/O for KV-separated trees
- `Destroy` leaves no tombstone -- only safe for keys written exactly once

---

## 10. KV Separation

Large values are stored in separate blob files, reducing compaction write amplification for blob-heavy workloads.

### KvSeparationOptions

```rust
let opts = KvSeparationOptions::default()
    .separation_threshold(1024)            // Values >= 1 KiB go to blob files. Default: 1 KiB.
    .file_target_size(64 * 1024 * 1024)    // Blob file target size. Default: 64 MiB.
    .staleness_threshold(0.25)             // GC trigger: 25% stale. Default: 0.25.
    .age_cutoff(0.25)                      // Age cutoff for GC. Default: 0.25.
    .compression(CompressionType::Lz4);    // Blob compression. Default: Lz4 (with feature).
```

### Configuration

```rust
let tree = db.keyspace("blobs", || {
    KeyspaceCreateOptions::default()
        .with_kv_separation(Some(
            KvSeparationOptions::default()
                .separation_threshold(4096)
                .file_target_size(128 * 1024 * 1024)
        ))
})?;
```

### Checking and Diagnostics

```rust
tree.is_kv_separated();             // bool
tree.fragmented_blob_bytes();       // Unreclaimed blob bytes
tree.blob_file_count();             // hidden
```

**How it works:**
- Values exceeding `separation_threshold` are written to blob files instead of SST data blocks
- An indirection pointer is stored in the LSM-tree pointing to the blob file location
- Blob GC reclaims space when staleness exceeds threshold
- Smaller blob files = more granular GC but more file handles
- Larger separation threshold = less compaction overhead but more I/O during compaction

---

## 11. Compression

### CompressionType

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CompressionType {
    None,
    Lz4,  // Only with "lz4" feature (default)
}
```

### CompressionPolicy (Per-Level)

```rust
// Disable compression on all levels
CompressionPolicy::disabled();

// Same compression on all levels
CompressionPolicy::all(CompressionType::Lz4);

// Per-level (index maps to LSM level)
CompressionPolicy::new([CompressionType::None, CompressionType::None, CompressionType::Lz4]);
```

### Default Policy

With the `lz4` feature (default):
- Journal: `Lz4` for values >= 4096 bytes
- Data blocks: `[None, None, Lz4]` -- Lz4 only on deeper levels
- Index blocks: `None` on all levels

### Usage

```rust
let opts = KeyspaceCreateOptions::default()
    .data_block_compression_policy(CompressionPolicy::new([
        CompressionType::None,
        CompressionType::Lz4,
    ]))
    .index_block_compression_policy(CompressionPolicy::disabled());
```

---

## 12. Bloom Filters

### FilterPolicy

```rust
// Same policy for all levels
FilterPolicy::all(FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)));

// Per-level policy
FilterPolicy::new([
    FilterPolicyEntry::Bloom(BloomConstructionPolicy::FalsePositiveRate(0.0001)),
    FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0)),
]);
```

### BloomConstructionPolicy

```rust
pub enum BloomConstructionPolicy {
    FalsePositiveRate(f64),  // e.g., 0.0001 for 0.01% FPR
    BitsPerKey(f64),         // e.g., 10.0 bits per key
}
```

### expect_point_read_hits

```rust
let opts = KeyspaceCreateOptions::default()
    .expect_point_read_hits(true);
```

When enabled, the last (largest) level does not build Bloom filters. This typically reduces filter space by ~90% since most data resides in the last level. Only enable if point reads are expected to find a key (i.e., very few "miss" lookups).

### Partitioned Filters

```rust
let opts = KeyspaceCreateOptions::default()
    .filter_block_partitioning_policy(PartitioningPolicy::new([false, false, false, true]));
```

Partitioned filters split the filter into blocks, allowing finer-grained loading. Default: partitioned on level 3+.

---

## 13. Bulk Loading

### Ingestion API

For maximum bulk load throughput, use the ingestion API which writes directly to SST files, bypassing the journal and memtable.

```rust
let mut ingestion = tree.start_ingestion()?;

ingestion.write("key1", "value1")?;
ingestion.write("key2", "value2")?;
ingestion.write_tombstone("deleted_key")?;

ingestion.finish()?;
```

### Ingestion Type

```rust
pub struct Ingestion<'a> {
    keyspace: &'a Keyspace,
    inner: AnyIngestion<'a>,
}

impl<'a> Ingestion<'a> {
    pub fn write<K: Into<UserKey>, V: Into<UserValue>>(&mut self, key: K, value: V) -> Result<()>;
    pub fn write_tombstone<K: Into<UserKey>>(&mut self, key: K) -> Result<()>;
    pub fn write_weak_tombstone<K: Into<UserKey>>(&mut self, key: K) -> Result<()>;  // hidden
    pub fn finish(self) -> Result<()>;
}
```

### Requirements

- **Keys MUST be written in ascending sorted order.** Panics if violated.
- The ingestion acquires the journal lock during `finish()` to prevent race conditions with concurrent writes.
- After `finish()`, a compaction hint is sent to background workers.
- Prefer ingestion over batch inserts for bulk data -- it is significantly faster.

---

## 14. Durability (PersistMode)

### PersistMode Enum

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PersistMode {
    /// Flushes to OS buffers. Survives app crash, NOT power loss.
    Buffer,

    /// fdatasync. Survives power loss (data only, no metadata).
    SyncData,

    /// fsync. Full durability (data + metadata).
    SyncAll,
}
```

### Usage

```rust
// Database-level persist
db.persist(PersistMode::SyncAll)?;

// Batch-level durability
let batch = db.batch()
    .durability(Some(PersistMode::SyncAll));
batch.commit()?;

// Transaction-level durability
let tx = db.write_tx()
    .durability(Some(PersistMode::SyncData));
```

### Default Behavior

When `manual_journal_persist` is `false` (default):
- Individual writes and batch commits automatically use `PersistMode::Buffer`
- On database drop, `PersistMode::SyncAll` is attempted

When `manual_journal_persist` is `true`:
- No automatic persist -- caller must use `db.persist()` explicitly
- Useful for ACID transactions where the caller controls when data hits disk

### Poisoned State

If any persist/flush operation fails (hardware error), the database enters a **poisoned** state:
- All future writes return `Error::Poisoned`
- This is a fatal, hardware-related failure
- The application should crash and recover
- See: <https://www.usenix.org/system/files/atc20-rebello.pdf>

---

## 15. Key Types and Errors

### Error

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Storage(lsm_tree::Error),
    Io(std::io::Error),
    JournalRecovery(JournalRecoveryError),
    InvalidVersion(Option<FormatVersion>),
    Decompress(CompressionType),
    InvalidTrailer,
    InvalidTag((&'static str, u8)),
    Poisoned,
    KeyspaceDeleted,
    Locked,
    Unrecoverable,
}

pub type Result<T> = std::result::Result<T, Error>;
```

Implements `std::error::Error` with `source()` chaining for `Storage`, `Io`, and `JournalRecovery` variants.

### JournalRecoveryError

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JournalRecoveryError {
    InsufficientLength,
    TooManyItems,
    ChecksumMismatch,
    InvalidFileName,
}
```

### KvPair

```rust
pub type KvPair = (UserKey, UserValue);  // (Slice, Slice)
```

### UserKey / UserValue

```rust
pub type UserKey = Slice;
pub type UserValue = Slice;
```

Keys: up to 65536 bytes. Values: up to 2^32 bytes.

### SeqNo

```rust
pub type SeqNo = u64;
```

Monotonically increasing sequence number. Used internally for MVCC visibility and version ordering. Accessible via `db.seqno()` (hidden) and `db.visible_seqno()` (hidden).

### FormatVersion

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FormatVersion {
    V1,  // 1.x.x releases
    V2,  // 2.x.x releases
    V3,  // 3.x.x releases (current)
}
```

File header: magic bytes `b"FJL"` + version byte (1 byte). Total: 4 bytes. Opening a V2 database returns `Error::InvalidVersion` with a migration tool recommendation.

### Additional Re-exports

From `lsm-tree`:
```rust
pub use lsm_tree::{
    CompressionType, KvPair, KvSeparationOptions, SeqNo, Slice,
    UserKey, UserValue,
};

// Hidden re-exports
pub use lsm_tree::{AbstractTree, AnyTree, Error as LsmError, TreeType};
```

### Compaction Strategies

```rust
pub mod compaction {
    pub use lsm_tree::compaction::{Fifo, Leveled, Levelled};

    // Leveled (default)
    // - L0 threshold: 4
    // - Table target size: 64 MiB
    // - Level ratio: 10x

    // Fifo
    // - Size limit + optional TTL
    // - Best for append-only, time-series, logs
}
```

### Config Policies Module

```rust
pub mod config {
    pub use lsm_tree::config::{
        BlockSizePolicy,
        BloomConstructionPolicy,
        CompressionPolicy,
        FilterPolicy,
        FilterPolicyEntry,
        HashRatioPolicy,
        PartitioningPolicy,
        PinningPolicy,
        RestartIntervalPolicy,
    };
}
```

### Utility Functions

```rust
pub mod util {
    pub use lsm_tree::util::{prefix_to_range, prefixed_range};
}
```
