# ShardedMap

A **thread-safe, concurrent hash map** library for Rust that uses sharding to minimize contention and maximize throughput in multi-threaded environments.

## Overview

ShardedMap distributes data across multiple independent **shards** (partitions), with each shard protected by its own lock. This design allows concurrent reads and writes on different shards without blocking each other, providing superior scalability compared to a single global lock.

### Key Features

- **🔒 Per-shard locking** — Each shard has its own RwLock, enabling concurrent access across shards
- **🔌 Pluggable architecture** — Swap between different map implementations (HashMap, BTreeMap, or custom)
- **🎯 Flexible lock mechanisms** — Easily swap the locking primitive (std::sync::RwLock or custom)
- **⚙️ Generic design** — Works with any `Hash + Eq` key and `Clone` value type

## Architecture

### Sharding Strategy

Data is distributed across multiple shards based on the hash of the key:

```rust
shard_index = hash(key) % num_shards
```

This ensures:
- Related keys map to the same shard deterministically
- Load is distributed evenly across shards
- Concurrent access to different shards proceeds independently

### Core Abstraction

ShardedMap uses trait-based abstractions for maximum flexibility:

- **`ShardableMap<K, V>`** — Defines map operations (get, insert, remove)
- **`ShardLock<T>`** — Defines locking behavior (read/write guards)

This allows you to:
1. **Choose your map implementation**: HashMap, BTreeMap, or custom
2. **Choose your lock mechanism**: std::sync::RwLock or custom implementations
3. **Add custom shard logic**: Extend traits for application-specific needs

## Usage

### Basic Example

```rust
use shardedmap::builder::Builder;

fn main() {
    // Create a sharded map with 16 shards using HashMap backend
    let map = Builder::new_default_lock_with_hashmap::<String, String>(16);

    // Insert values
    map.insert("email".to_string(), "user@example.com".to_string());
    map.insert("name".to_string(), "Alice".to_string());

    // Retrieve values
    if let Some(email) = map.get(&"email".to_string()) {
        println!("Email: {}", email);
    }
}
```

### Using BTreeMap Backend

```rust
use shardedmap::builder::Builder;

let map = Builder::new_default_lock_with_btreemap::<i32, String>(32);
map.insert(1, "one".to_string());
map.insert(2, "two".to_string());
```

### Using Custom Map Implementation

```rust
use shardedmap::builder::Builder;
use shardedmap::custommap::CustomMap;

let map = Builder::new_default_lock_with_custom_map::<String, String, CustomMap<String, String>>(16);
map.insert("key".to_string(), "value".to_string());
```

## Supported Map Backends

| Backend | Characteristics | Best For |
|---------|-----------------|----------|
| **HashMap** | O(1) average case, unordered | General-purpose, high-performance |
| **BTreeMap** | O(log n), ordered, requires `Ord` | Range queries, sorted iteration |
| **CustomMap** | Separate chaining, 100 buckets | Educational, learning Rust internals |

## Performance Considerations

### Advantages
- **Reduced contention** — Locks apply only to individual shards, not the entire map
- **Scalability** — Throughput increases with the number of CPU cores (up to the number of shards)
- **Concurrent reads** — Multiple readers can access different shards simultaneously

### Trade-offs
- **Memory overhead** — Each shard requires its own map and lock instance
- **Shard count tuning** — Too few shards = contention; too many = memory waste
- **Hash function performance** — Every access requires hashing the key

### Optimal Shard Count
- **General guideline**: 2x to 4x the number of CPU cores
- **Low contention workload**: Fewer shards (reduce memory)
- **High contention workload**: More shards (increase parallelism)

## API Reference

### Constructor Methods

```rust
// Create with HashMap backend
Builder::new_default_lock_with_hashmap::<K, V>(num_shards) -> ShardedMap<K, V, HashMap, RwLock>

// Create with BTreeMap backend
Builder::new_default_lock_with_btreemap::<K, V>(num_shards) -> ShardedMap<K, V, BTreeMap, RwLock>

// Create with custom map backend
Builder::new_default_lock_with_custom_map::<K, V, M>(num_shards) -> ShardedMap<K, V, M, RwLock>
```

### Core Methods

```rust
// Insert or update a key-value pair
map.insert(key: K, value: V) -> ()

// Retrieve a value (read-only)
map.get(key: &K) -> Option<V>
```

## Generic Parameters

ShardedMap is generic over 4 types:

```rust
ShardedMap<K, V, M, L>
```

- **K** — Key type (must implement `Hash + Eq`)
- **V** — Value type (must implement `Clone`)
- **M** — Map implementation (must implement `ShardableMap<K, V>`)
- **L** — Lock type (must implement `ShardLock<M>`)

## Benchmarks

Benchmarked against [DashMap](https://github.com/xacrimon/dashmap) and a naive `Mutex<HashMap>` baseline with Criterion across 1–16 threads (16 shards):

- **~3.1x higher throughput** than `Mutex<HashMap>` at 16 threads (10.10ms vs 31.35ms)
- **~1.7x slower than DashMap** at 16 threads (10.10ms vs 5.78ms)
- **HashMap vs BTreeMap backends perform nearly identically at scale** (10.10ms vs 10.65ms at 16 threads) — lock contention dominates
- Shard count confirmed as a tuning tradeoff: 1 shard costs 3.6x more than 64 shards at 8 threads (16.12ms vs 4.51ms)

Full results, methodology, system specs, and how to read the raw reports: [BENCHMARKS.md](./BENCHMARKS.md)

## License

See [LICENSE](LICENSE) file for details.
