use criterion::{criterion_group, criterion_main, Criterion};
use std::sync::{Arc, Mutex};
use std::thread;
use trades_subsystem::portfolio::Portfolio;
use trades_subsystem::decision::decide_action;

fn benchmark_concurrent_trades(c: &mut Criterion) {
    c.bench_function("concurrent_trades", |b| {
        b.iter(|| {
            let portfolio = Arc::new(Mutex::new(Portfolio::new(10_000.0)));
            let mut handles = vec![];
            for _ in 0..100 {
                let portfolio_ref = Arc::clone(&portfolio);
                handles.push(thread::spawn(move || {
                    let portfolio = portfolio_ref.lock().unwrap();
                    decide_action(&portfolio, "AAPL", 150.0, 100);
                }));
            }
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

criterion_group!(benches, benchmark_concurrent_trades);
criterion_main!(benches);
