use criterion::{criterion_group, criterion_main, Criterion, black_box};
use trades_subsystem::portfolio::Portfolio;
use trades_subsystem::decision::decide_action;
use rand::Rng;

fn benchmark_decision(c: &mut Criterion) {
    c.bench_function("decision_bench_multiple_stocks", |b| {
        let portfolio = Portfolio::new(10000.0);
        let stocks = vec![
            "AAPL", "GOOGL", "AMZN", "META", "MSFT", "TSLA", "NFLX", "NVDA",
            "BABA", "ORCL", "INTC", "CSCO", "ADBE", "IBM", "PYPL"
        ];
        let mut rng = rand::thread_rng();

        b.iter(|| {
            for stock_id in &stocks {
                let price = rng.gen_range(50.0..300.0); // Aligning with producer price range
                let quantity = rng.gen_range(5..15); // Aligning with producer quantity range
                decide_action(
                    black_box(&portfolio),
                    black_box(stock_id),
                    black_box(price),
                    black_box(quantity),
                );
            }
        });
    });
}

criterion_group!(decision_bench, benchmark_decision);
criterion_main!(decision_bench);
