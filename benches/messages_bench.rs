use criterion::{criterion_group, criterion_main, Criterion};
use amiquip::{Connection, Exchange, Publish, QueueDeclareOptions, ConsumerOptions, ConsumerMessage};

fn benchmark_message_passing(c: &mut Criterion) {
    c.bench_function("rabbitmq_message_latency", |b| {
        b.iter(|| {
            let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672").unwrap();
            let channel = connection.open_channel(None).unwrap();
            let exchange = Exchange::direct(&channel);
            let queue = channel.queue_declare("test_queue", QueueDeclareOptions::default()).unwrap();
            let consumer = queue.consume(ConsumerOptions::default()).unwrap();
            let start_time = std::time::Instant::now();
            exchange.publish(Publish::new(b"Test Message", "test_queue")).unwrap();

            match consumer.receiver().recv().unwrap() {
                ConsumerMessage::Delivery(delivery) => {
                    consumer.ack(delivery).unwrap();
                    let elapsed = start_time.elapsed();
                    println!("Latency: {:.2?}", elapsed);
                }
                _ => {}
            }

            connection.close().unwrap();
        });
    });
}

criterion_group!(benches, benchmark_message_passing);
criterion_main!(benches);
