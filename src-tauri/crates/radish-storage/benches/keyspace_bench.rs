use criterion::{criterion_group, criterion_main, Criterion};
use bytes::Bytes;
use radish_storage::{Keyspace, Value};

fn bench_set(c: &mut Criterion) {
    let mut ks = Keyspace::new();
    let val = Value::String(Bytes::from("hello world"));

    c.bench_function("keyspace set", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = Bytes::from(format!("key-{}", i));
            ks.set(key, val.clone());
            i += 1;
        })
    });
}

fn bench_get(c: &mut Criterion) {
    let mut ks = Keyspace::new();
    let val = Value::String(Bytes::from("hello world"));
    
    // Pre-populate
    for i in 0..10_000 {
        ks.set(Bytes::from(format!("key-{}", i)), val.clone());
    }

    c.bench_function("keyspace get", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key-{}", i % 10_000);
            criterion::black_box(ks.get(key.as_bytes()));
            i += 1;
        })
    });
}

fn bench_snapshot(c: &mut Criterion) {
    let mut ks = Keyspace::new();
    let val = Value::String(Bytes::from("hello world"));
    
    // Pre-populate with 10k items
    for i in 0..10_000 {
        ks.set(Bytes::from(format!("key-{}", i)), val.clone());
    }

    c.bench_function("keyspace snapshot 10k", |b| {
        b.iter(|| {
            criterion::black_box(ks.snapshot());
        })
    });
}

criterion_group!(benches, bench_set, bench_get, bench_snapshot);
criterion_main!(benches);
