use chrono::Utc;
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use crate::models::TradeMessage;

pub fn simulate_price_updates(
    trade_message_sender: mpsc::Sender<TradeMessage>,
    simulation_duration_secs: u64,
) {
    let stocks = vec![
        "AAPL", "GOOGL", "AMZN", "META", "MSFT", "TSLA", "NFLX", "NVDA", 
        "BABA", "ORCL", "INTC", "CSCO", "ADBE", "IBM", "PYPL",
        "V", "AMD", "QCOM", "INTU", "CRM", "UBER", "LYFT", "SNAP", 
        "TWTR", "SQ", "ZM", "SHOP", "ASML", "TXN", "ADSK",
        "JPM", "BAC", "C", "GS", "AXP", "VZ", "SCHW",
        "WMT", "DIS", "MCD", "NKE", "LOW", "HD", "SBUX", "TGT", "ROKU",
        "PFE", "MRK", "JNJ", "UNH", "CVS", "GILD", "AMGN", "BMY", "SNY",
        "XOM", "CVX", "BA", "GE"
    ];
    
    let mut rng = rand::thread_rng();
    let simulation_start_time = Instant::now();
    let simulation_duration = Duration::from_secs(simulation_duration_secs);
    let trend = rng.gen_range(-0.02..0.02);

    println!("[Producer] Starting price simulation for {} seconds.", simulation_duration_secs);

    while Instant::now() - simulation_start_time < simulation_duration {
        for stock_id in &stocks {
            let old_price = rng.gen_range(50.0..500.0);
            let new_price = old_price * (1.0 + rng.gen_range(-0.10..0.10) + trend);
            let price_change = new_price - old_price;
            let quantity = rng.gen_range(5..15);

            let message = TradeMessage {
                stock_id: stock_id.to_string(),
                current_price: new_price,
                action_type: "PRICE_UPDATE".to_string(),
                quantity,
                timestamp: Utc::now().to_rfc3339(),
            };

            println!(
                "[Producer] Updated Price for {:<5}: ${:.2} -> ${:.2} (Change: {:.2}) at {}",
                stock_id, old_price, new_price, price_change, message.timestamp
            );

            if let Err(e) = trade_message_sender.send(message) {
                eprintln!("[ERROR] Failed to send trade message for {}: {}", stock_id, e);
            }

            thread::sleep(Duration::from_millis(300)); 
        }
    }

    println!("[Producer] Simulation completed after {} seconds.", simulation_duration_secs);
}
