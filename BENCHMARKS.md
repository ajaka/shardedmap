# Benchmarks

All benchmarks run with [Criterion](https://github.com/bheisler/criterion.rs). Each concurrent benchmark spawns N threads, each performing 1,000 insert+get pairs on disjoint key ranges, and measures total wall-clock time for all threads to complete. Raw reports (HTML + JSON) are in `criterion/`; run `cargo bench` to reproduce. Criterion also generates interactive HTML reports with violin plots and regression lines for each benchmark — after running `cargo bench`, open `criterion/report/index.html` in a browser for the full visual breakdown.

## System Specifications

| Component | Details                                                      |
| --------- | ------------------------------------------------------------ |
| **CPU**   | Intel Core i5-6300U @ 2.40GHz (2 cores, 4 threads, 3 MiB L3) |
| **RAM**   | 7.6 GiB DDR4                                                 |
| **OS**    | Linux 7.0.0-30-generic (Ubuntu)                              |
| **Rust**  | 1.82+ (stable)                                               |
| **Build** | `cargo bench --release`                                      |

> **Note**: Results are specific to this hardware. On different CPUs (more cores, different microarchitecture, different cache hierarchy), absolute numbers and scaling behavior will vary. The _relative_ ordering (DashMap > ShardedMap > Mutex<HashMap>) is expected to hold, but the gap magnitudes may change.

## ShardedMap vs. DashMap vs. Mutex\<HashMap\>

ShardedMap (16 shards) benchmarked against [DashMap](https://github.com/xacrimon/dashmap) and a naive `Mutex<HashMap>` baseline, across 1–16 threads. Two ShardedMap backends tested: **HashMap** and **BTreeMap**.

| Threads | ShardedMap (HashMap) | ShardedMap (BTreeMap) | DashMap | Mutex\<HashMap\> |
| ------: | -------------------: | --------------------: | ------: | ---------------: |
|       1 |              0.91 ms |               0.93 ms | 0.69 ms |          0.76 ms |
|       2 |              1.63 ms |               1.57 ms | 1.11 ms |          2.22 ms |
|       4 |              2.95 ms |               3.01 ms | 1.85 ms |          7.11 ms |
|       8 |              5.13 ms |               5.33 ms | 3.10 ms |         14.57 ms |
|      16 |             10.10 ms |              10.65 ms | 5.78 ms |         31.35 ms |

**Takeaways:**

- `Mutex<HashMap>` scales poorly — time roughly doubles every doubling of threads, the expected signature of a single global lock fully serializing all access.
- Both ShardedMap and DashMap scale sub-linearly: per-thread cost drops as thread count rises, since work spreads across independent locks instead of queuing on one.
- **ShardedMap (HashMap) delivers ~3.1x higher throughput than `Mutex<HashMap>` at 16 threads** (10.10ms vs 31.35ms).
- **DashMap is ~1.7x faster than ShardedMap (HashMap) at 16 threads** (5.78ms vs 10.10ms). This gap is **not** due to trait-dispatch overhead (see "Why the Gap?" below).
- **ShardedMap (BTreeMap) matches ShardedMap (HashMap) at high thread counts** (10.65ms vs 10.10ms at 16 threads), despite BTreeMap's O(log n) complexity — the lock contention dominates at scale.
- At 1 thread, `Mutex<HashMap>` and DashMap are faster than ShardedMap — the sharding machinery (hashing to shard index, vector indirection) adds fixed overhead with no parallelism benefit.

### Why the Gap vs DashMap?

The reasons DashMap is faster:

1. **Single hash per operation** — DashMap hashes once; ShardedMap hashes once for shard index, then the inner `HashMap` hashes again.
2. **Finer-grained locking** — DashMap uses a custom sharded array with per-shard `RwLock` _and_ per-bucket locking in some configurations; ShardedMap uses one `RwLock` per entire shard.
3. **Memory layout** — DashMap's `Vec<Bucket>` stores entries inline; `std::collections::HashMap` has an extra pointer indirection per entry.
4. **Hash function** — DashMap defaults to `ahash` (fast, randomized); std `HashMap` uses SipHash (DoS-resistant, slower).
5. **Optimization maturity** — DashMap has years of micro-optimizations (prefetching, lock elision, etc.).

If you need DashMap-level performance, use DashMap. ShardedMap's value is **pluggable backends** (HashMap, BTreeMap, custom) and **lock flexibility** — not raw speed.

## Shard Count Tuning (`concurrent_mixed`)

Isolating shard count as an independent variable (1/4/16/64 shards) across thread counts, holding everything else fixed (HashMap backend, 1000 ops/thread):

| Threads |  1 shard | 4 shards | 16 shards | 64 shards |
| ------: | -------: | -------: | --------: | --------: |
|       1 |  0.77 ms |  0.93 ms |   0.90 ms |   0.95 ms |
|       2 |  2.95 ms |  1.99 ms |   1.63 ms |   1.40 ms |
|       4 |  7.66 ms |  4.23 ms |   2.97 ms |   2.37 ms |
|       8 | 16.12 ms |  8.19 ms |   5.68 ms |   4.51 ms |

**Takeaways:**

- At **1 thread (no contention possible)**, shard count makes little difference — all configs within ~20%. The slight edge for 1 shard (0.77ms vs 0.95ms) is the expected overhead of hashing to a shard index and maintaining more lock instances.
- At **2+ threads**, the picture flips dramatically: the 1-shard config degrades toward a single global lock (2.95ms at 2 threads, 16.12ms at 8), while 64 shards absorbs the load (1.40ms at 2 threads, 4.51ms at 8).
- This confirms the core design tradeoff: shard count is a real dial between single-threaded overhead and multi-threaded scalability.
- **Practical guideline**: 2×–4× CPU cores for high-contention workloads; fewer shards for low-contention or memory-constrained scenarios.

## Single-Threaded Insert Overhead (HashMap Backend)

| Shard Count | Mean Insert Time |
| ----------: | ---------------: |
|           1 |           587 ns |
|           8 |           583 ns |
|          64 |           590 ns |

No meaningful difference (within noise) — sharding overhead is negligible for single-threaded workloads at this key/value size.

## HashMap vs. BTreeMap Backend (Single-Key Get)

| Backend  | Mean Get Time |
| -------- | ------------: |
| HashMap  |        183 ns |
| BTreeMap |        120 ns |

BTreeMap's `get` outperforms HashMap's here — but this is an artifact of the benchmark using a single stored key (n=1). Rust's default `HashMap` hasher (SipHash) pays a fixed, DoS-resistant hashing cost on every lookup regardless of map size, while BTreeMap on a single-entry tree does effectively zero comparisons. At larger n, HashMap's O(1) average case would be expected to win. Included for completeness, not as a backend recommendation.

## Reading the Raw Reports

- `mean` vs. `median`: if `mean` is noticeably higher than `median`, a handful of runs were slow outliers (scheduling noise, thread preemption) — look at `report/pdf.svg` (violin plot) to confirm a long right tail rather than a shifted distribution.
- Coefficient of variation (`std_dev / mean`) above ~10% on these benchmarks generally indicates environmental noise rather than a real property of the code, particularly at low thread counts where total work per run is small.
- Criterion's "change" detection compares against the previous run's `base/` estimates — "Performance has regressed/improved" means statistically significant change vs. last run, not vs. other implementations.
