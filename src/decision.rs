use crate::portfolio::{Portfolio, FEE_RATE};

const DEBUG_ENABLED: bool = false;

//Broker
pub fn decide_action(
    portfolio: &Portfolio,
    stock_id: &str,
    price: f64,
    incoming_qty: u32,
) -> (&'static str, Option<u32>) {
    if portfolio.balance < 0.0 {
        eprintln!("[ERROR] Invalid portfolio state: negative balance (${:.2}).", portfolio.balance);
        return ("REFUSE", None);
    }

    let last_price = portfolio.last_prices.get(stock_id).cloned().unwrap_or(price); 
    let (quantity, avg_cost) = portfolio.holdings.get(stock_id).cloned().unwrap_or((0, 0.0));

    let buy_threshold = 0.95;  
    let max_buy_pct = 0.10;  
    let sell_threshold_high = 1.10;  
    let sell_threshold_low = 1.05;   
    let stop_loss_threshold = 0.90;  
    let trailing_stop_threshold = 0.07; 

    let max_shares_to_buy = ((portfolio.balance * max_buy_pct) / (price * (1.0 + FEE_RATE))) as u32;

    if quantity == 0 || price < last_price * buy_threshold {
        let affordable_shares = ((portfolio.balance / (price * (1.0 + FEE_RATE))) as u32).max(0);
        let buy_qty = affordable_shares.min(incoming_qty).min(max_shares_to_buy);  

        if buy_qty > 0 {
            if DEBUG_ENABLED {
                println!("[DEBUG] Buy Decision: Stock: {}, Quantity: {}, Price: {:.2}", stock_id, buy_qty, price);
            }
            return ("BUY", Some(buy_qty));
        } else {
            println!("[Consumer] Could not buy {} at ${:.2} due to insufficient funds.", stock_id, price);
            return ("REFUSE", None);
        }
    }
    else if quantity > 0 {
        let potential_profit = (price - avg_cost) / avg_cost;
        let mut peak_price = portfolio.last_prices.get(stock_id).cloned().unwrap_or(price);
        peak_price = peak_price.max(price); 

        let trailing_stop_price = peak_price * (1.0 - trailing_stop_threshold);

        if price > avg_cost * sell_threshold_high {
            let sell_qty = (quantity as f64 * 0.75) as u32;
            if DEBUG_ENABLED {
                println!("[DEBUG] Aggressive Sell Decision: Stock: {}, Quantity: {}, Price: {:.2}, Profit: {:.2}%", stock_id, sell_qty, price, potential_profit * 100.0);
            }
            return ("SELL", Some(sell_qty));
        }
        else if price > avg_cost * sell_threshold_low {
            let sell_qty = (quantity as f64 * 0.50) as u32;  
            if DEBUG_ENABLED {
                println!("[DEBUG] Partial Sell Decision: Stock: {}, Quantity: {}, Price: {:.2}, Profit: {:.2}%", stock_id, sell_qty, price, potential_profit * 100.0);
            }
            return ("SELL", Some(sell_qty));
        }
        else if price < avg_cost * stop_loss_threshold {
            if DEBUG_ENABLED {
                println!("[DEBUG] Stop Loss Triggered: Stock: {}, Price: {:.2}, Cost: {:.2}", stock_id, price, avg_cost);
            }
            return ("SELL", Some(quantity));  
        }
        else if price < trailing_stop_price {
            if DEBUG_ENABLED {
                println!("[DEBUG] Trailing Stop Triggered: Stock: {}, Price: {:.2}, Peak: {:.2}, Trailing Stop: {:.2}", stock_id, price, peak_price, trailing_stop_price);
            }
            return ("SELL", Some(quantity));  
        }
        else {
            println!("[Consumer] No action taken for {} at ${:.2} because Unfavorable Price.", stock_id, price);
            return ("REFUSE", None);
        }
    }
    else {
        println!("[Consumer] No action taken for {} at ${:.2} because Unfavorable Price.", stock_id, price);
        return ("REFUSE", None);
    }
}

//Order Manager
pub fn execute_trade_action(
    portfolio: &mut Portfolio,
    stock_id: &str,
    price: f64,
    action: &str,
    final_quantity: Option<u32>,
) -> Option<u32> {
    if let Some(quantity) = final_quantity {
        if DEBUG_ENABLED {
            println!(
                "[DEBUG] Executing Action: {}, Stock: {}, Quantity: {}, Price: {:.2}",
                action, stock_id, quantity, price
            );
        }

        match action {
            "BUY" => {
                let cost = price * quantity as f64 * (1.0 + FEE_RATE);
                if portfolio.balance >= cost {
                    portfolio.update(stock_id, quantity, price, action);
                    Some(quantity)
                } else {
                    println!(
                        "[Order Manager] Consumer don't have insufficient funds to complete BUY of {} shares of {}.",
                        quantity, stock_id
                    );
                    None
                }
            }
            "SELL" => {
                if portfolio.get_stock_quantity(stock_id) >= quantity {
                    portfolio.update(stock_id, quantity, price, action);
                    Some(quantity)
                } else {
                    println!(
                        "[Order Manager] Consumer do not have enough shares to SELL {} shares of {}.",
                        quantity, stock_id
                    );
                    None
                }
            }
            _ => {
                println!("[ERROR] Unknown action: {}. No trade executed.", action);
                None
            }
        }
    } else {
        if DEBUG_ENABLED {
            println!(
                "[DEBUG] No Action Executed: Stock: {}, Action: {}, Quantity: None",
                stock_id, action
            );
        }
        None
    }
}

/*
#[test]
fn test_decide_action_buy() {
    let mut portfolio = Portfolio::new(10000.0);
    portfolio.update_last_price("AAPL", 160.0);
    let action = decide_action(&portfolio, "AAPL", 150.0, 100);
    assert_eq!(action.0, "BUY");
}
*/