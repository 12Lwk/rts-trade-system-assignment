//extra simulate_price_updates
use criterion::{criterion_group, criterion_main, Criterion};
use trades_subsystem::producer::simulate_price_updates;
use std::sync::mpsc;

fn benchmark_producer(c: &mut Criterion) {
    c.bench_function("simulate_price_updates", |b| {
        let (sender, _receiver) = mpsc::channel();
        b.iter(|| simulate_price_updates(sender.clone(), 1)); // Simulate for 1 second
    });
}

criterion_group!(benches, benchmark_producer);
criterion_main!(benches);