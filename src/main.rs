mod models; 
mod portfolio;
mod producer;
mod decision;

use amiquip::{Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions, Result};
use chrono::Utc;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use models::{TradeMessage, TradeResponse};
use portfolio::Portfolio;
use producer::simulate_price_updates;
use decision::{decide_action, execute_trade_action};

fn main() -> Result<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);
    let queue = channel.queue_declare("trade_queue", QueueDeclareOptions::default())?;

    let portfolio = Arc::new(Mutex::new(Portfolio::new(10000.0)));

    let (price_update_sender, price_update_receiver) = mpsc::channel();
    let (trade_response_sender, trade_response_receiver) = mpsc::channel();

    let price_update_channel = price_update_sender.clone();
    let simulation_duration_secs = 180;

    thread::spawn(move || simulate_price_updates(price_update_channel, simulation_duration_secs));

    println!("\nStarting simulation for {} seconds...\n", simulation_duration_secs);
    let simulation_start_time = Instant::now();
    let simulation_duration = Duration::from_secs(simulation_duration_secs);

    let rabbitmq_consumer = queue.consume(ConsumerOptions::default())?;

    while Instant::now() - simulation_start_time < simulation_duration {
        if let Ok(message) = price_update_receiver.try_recv() {
            match serde_json::to_string(&message) {
                Ok(payload) => {
                    println!(
                        "[Producer] Updated Price for {:<5}: ${:.2} at Quantity: {} at {}",
                        message.stock_id, message.current_price, message.quantity, message.timestamp
                    );
                    if let Err(e) = exchange.publish(Publish::new(payload.as_bytes(), "trade_queue")) {
                        eprintln!("[ERROR] Failed to publish message: {}", e);
                    }
                }
                Err(e) => eprintln!("[ERROR] Failed to serialize message: {}", e),
            }
        }

        match rabbitmq_consumer.receiver().try_recv() {
            Ok(ConsumerMessage::Delivery(rabbitmq_message)) => {
                let payload = String::from_utf8_lossy(&rabbitmq_message.body);
                match serde_json::from_str::<TradeMessage>(&payload) {
                    Ok(trade_message) => {
                        let portfolio_ref = Arc::clone(&portfolio);
                        let tx_response = trade_response_sender.clone();
                        thread::spawn(move || {
                            let mut portfolio = portfolio_ref.lock().unwrap();

                            let (action, final_quantity) = decide_action(
                                &portfolio,
                                &trade_message.stock_id,
                                trade_message.current_price,
                                trade_message.quantity,
                            );

                            let executed_quantity = execute_trade_action(
                                &mut portfolio,
                                &trade_message.stock_id,
                                trade_message.current_price,
                                &action,
                                final_quantity,
                            );

                            portfolio.update_last_price(&trade_message.stock_id, trade_message.current_price);

                            let response = TradeResponse {
                                stock_id: trade_message.stock_id,
                                decision: action.to_string(),
                                quantity: executed_quantity.unwrap_or(0),
                                price: trade_message.current_price,
                                timestamp: Utc::now().to_rfc3339(),
                            };

                            if let Err(e) = tx_response.send(response) {
                                eprintln!("[ERROR] Failed to send trade response: {}", e);
                            }
                        });
                    }
                    Err(e) => eprintln!("[ERROR] Failed to parse incoming message: {}", e),
                }
                rabbitmq_consumer.ack(rabbitmq_message)?;
            }
            Err(_) => thread::sleep(Duration::from_millis(100)),
            _ => {}
        }

        if let Ok(response) = trade_response_receiver.try_recv() {
            println!(
                "[Producer] Trade Decision Received: Stock: {}, Action: {}, Quantity: {}, Price: ${:.2}, Timestamp: {}\n",
                response.stock_id,
                response.decision,
                response.quantity,
                response.price,
                response.timestamp
            );

            let response_exchange = Exchange::direct(&channel);

            if let Ok(payload) = serde_json::to_string(&response) {
                if let Err(e) = response_exchange.publish(Publish::new(payload.as_bytes(), "trade_response_queue")) {
                    eprintln!("[ERROR] Failed to publish trade response: {}", e);
                }
            } else {
                eprintln!("[ERROR] Failed to serialize trade response.");
            }
        }
    }

    println!("\nSimulation completed. Final Portfolio:\n");
    {
        let portfolio = portfolio.lock().unwrap();
        portfolio.display_summary();
    }

    connection.close()?;
    Ok(())
}
