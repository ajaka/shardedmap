use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use dashmap::DashMap;
use shardedmap::Builder;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

const OPS_PER_THREAD: u64 = 1000;

fn bench_concurrent_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_map_comparison");

    for threads in [1, 2, 4, 8, 16] {
        // ShardedMap with HashMap backend
        group.bench_with_input(
            BenchmarkId::new("shardedmap_hashmap", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(Builder::new_default_lock_with_hashmap::<u64, u64>(16));
                    let handles: Vec<_> = (0..threads)
                        .map(|t| {
                            let map = Arc::clone(&map);
                            thread::spawn(move || {
                                for i in 0..OPS_PER_THREAD {
                                    let key = (t as u64) * OPS_PER_THREAD + i;
                                    map.insert(key, key);
                                    black_box(map.get(&key));
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );

        // ShardedMap with BTreeMap backend
        group.bench_with_input(
            BenchmarkId::new("shardedmap_btreemap", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(Builder::new_default_lock_with_btreemap::<u64, u64>(16));
                    let handles: Vec<_> = (0..threads)
                        .map(|t| {
                            let map = Arc::clone(&map);
                            thread::spawn(move || {
                                for i in 0..OPS_PER_THREAD {
                                    let key = (t as u64) * OPS_PER_THREAD + i;
                                    map.insert(key, key);
                                    black_box(map.get(&key));
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );

        // DashMap
        group.bench_with_input(
            BenchmarkId::new("dashmap", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(DashMap::<u64, u64>::new());
                    let handles: Vec<_> = (0..threads)
                        .map(|t| {
                            let map = Arc::clone(&map);
                            thread::spawn(move || {
                                for i in 0..OPS_PER_THREAD {
                                    let key = (t as u64) * OPS_PER_THREAD + i;
                                    map.insert(key, key);
                                    black_box(map.get(&key));
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );

        // Naive baseline: single global Mutex<HashMap>
        group.bench_with_input(
            BenchmarkId::new("mutex_hashmap", threads),
            &threads,
            |b, &threads| {
                b.iter(|| {
                    let map = Arc::new(Mutex::new(HashMap::<u64, u64>::new()));
                    let handles: Vec<_> = (0..threads)
                        .map(|t| {
                            let map = Arc::clone(&map);
                            thread::spawn(move || {
                                for i in 0..OPS_PER_THREAD {
                                    let key = (t as u64) * OPS_PER_THREAD + i;
                                    map.lock().unwrap().insert(key, key);
                                    black_box(map.lock().unwrap().get(&key).copied());
                                }
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_concurrent_comparison);
criterion_main!(benches);
