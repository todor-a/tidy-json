use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::Value;

use tidy_json::sort::sort;
use tidy_json::SortOrder;

/// Generate a flat JSON object with n keys
fn generate_flat_json(num_keys: usize) -> Value {
    let mut map = serde_json::Map::new();
    for i in 0..num_keys {
        let key = format!("key_{:05}", num_keys - i); // Reverse order to ensure sorting does work
        map.insert(key, Value::Number(i.into()));
    }
    Value::Object(map)
}

/// Generate a nested JSON object with specified depth and keys per level
fn generate_nested_json(depth: usize, keys_per_level: usize) -> Value {
    if depth == 0 {
        return Value::Number(1.into());
    }

    let mut map = serde_json::Map::new();
    for i in 0..keys_per_level {
        let key = format!("key_{}", keys_per_level - i);
        map.insert(key, generate_nested_json(depth - 1, keys_per_level));
    }
    Value::Object(map)
}

/// Benchmark sorting by JSON size (number of keys)
fn bench_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_size");

    for size in [10, 100, 500, 1000] {
        let json = generate_flat_json(size);

        group.bench_with_input(BenchmarkId::new("keys", size), &json, |b, json| {
            b.iter(|| sort(black_box(json), &SortOrder::AlphabeticalAsc, 0, None))
        });
    }

    group.finish();
}

/// Benchmark sorting by nesting depth
fn bench_by_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_depth");

    for depth in [1, 3, 5, 7] {
        let json = generate_nested_json(depth, 5);

        group.bench_with_input(BenchmarkId::new("depth", depth), &json, |b, json| {
            b.iter(|| sort(black_box(json), &SortOrder::AlphabeticalAsc, 0, None))
        });
    }

    group.finish();
}

/// Benchmark different sort orders
fn bench_by_sort_order(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_order");

    let json = generate_flat_json(100);

    let orders = [
        ("alphabetical_asc", SortOrder::AlphabeticalAsc),
        ("alphabetical_desc", SortOrder::AlphabeticalDesc),
        ("key_length_asc", SortOrder::KeyLengthAsc),
        ("key_length_desc", SortOrder::KeyLengthDesc),
    ];

    for (name, order) in orders {
        group.bench_with_input(BenchmarkId::new("order", name), &json, |b, json| {
            b.iter(|| sort(black_box(json), &order, 0, None))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_by_size, bench_by_depth, bench_by_sort_order);
criterion_main!(benches);
