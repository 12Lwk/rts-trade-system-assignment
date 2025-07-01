use std::collections::HashMap;

pub const FEE_RATE: f64 = 0.001;

pub struct Portfolio {
    pub holdings: HashMap<String, (u32, f64)>,
    pub balance: f64,
    pub cash_flow: f64,
    pub total_fees: f64,
    pub revenue: f64,
    pub total_cost: f64,
    pub last_prices: HashMap<String, f64>,
}

impl Portfolio {
    pub fn new(initial_balance: f64) -> Self {
        Portfolio {
            holdings: HashMap::new(),
            balance: initial_balance,
            cash_flow: 0.0,
            total_fees: 0.0,
            revenue: 0.0,
            total_cost: 0.0,
            last_prices: HashMap::new(),
        }
    }

    pub fn update(&mut self, stock_id: &str, quantity: u32, price: f64, action: &str) {
        if quantity == 0 || price < 0.0 {
            println!(
                "[INFO] Transaction not processed: Quantity {} must be greater than zero, and price ${:.2} must be non-negative. Action: {}.",
                quantity, price, action
            );
            return;
        }

        if self.balance < 0.0 {
            println!(
                "[INFO] Action paused: Portfolio balance is -${:.2}. Please review your financial standing before performing further actions.",
                self.balance
            );
            return;
        }

        let fee = FEE_RATE * price * quantity as f64; 
        match action {
            "BUY" => {
                let cost = price * quantity as f64 + fee;
                if self.balance >= cost {
                    let entry = self.holdings.entry(stock_id.to_string()).or_insert((0, 0.0));
                    let new_total_qty = entry.0 + quantity;
                    let new_avg_cost = ((entry.0 as f64 * entry.1) + (quantity as f64 * price)) / new_total_qty as f64;

                    entry.0 = new_total_qty;
                    entry.1 = new_avg_cost;
                    self.balance -= cost;
                    self.total_cost += cost;
                    self.total_fees += fee;
                    self.cash_flow -= cost;
                    println!(
                        "[Consumer] Bought {} shares of {} at ${:.2} - Cost: ${:.2} (incl. ${:.2} fee).",
                        quantity, stock_id, price, cost, fee
                    );
                } else {
                    println!(
                        "[Consumer] Insufficient funds to buy {} shares of {} - Need: ${:.2}, Have: ${:.2}.",
                        quantity, stock_id, cost, self.balance
                    );
                }
            }
            "SELL" => {
                let entry = self.holdings.entry(stock_id.to_string()).or_insert((0, 0.0));
                if entry.0 >= quantity {
                    let revenue = price * quantity as f64 - fee;
                    entry.0 -= quantity;
                    self.balance += revenue;
                    self.revenue += revenue;
                    self.total_fees += fee;
                    self.cash_flow += revenue;
                    println!(
                        "[Consumer] Sold {} shares of {} at ${:.2} - Revenue: ${:.2} (incl. ${:.2} fee).",
                        quantity, stock_id, price, revenue, fee
                    );
                } else {
                    println!(
                        "[Consumer] Not enough shares to sell {} of {} - Owned: {}.",
                        quantity, stock_id, entry.0
                    );
                }
            }
            _ => {
                println!("[ERROR] Skip Unknown action '{}'.", action);
            }
        }
    }

    pub fn update_last_price(&mut self, stock_id: &str, price: f64) {
        if price < 0.0 {
            println!(
                "[ERROR] Ignored invalid price ({:.2}) for {}.",
                price, stock_id
            );
            return;
        }
        self.last_prices.insert(stock_id.to_string(), price);
    }

    pub fn get_stock_quantity(&self, stock_id: &str) -> u32 {
        self.holdings.get(stock_id).map(|(qty, _)| *qty).unwrap_or(0)
    }

    pub fn display_summary(&self) {
        if self.balance < 0.0 {
            println!("[Info] Portfolio has negative balance.");
        }

        let initial_cash = self.balance + self.total_cost - self.revenue;
        println!("\n--- Portfolio Summary ---\n");
        println!("Initial Cash:        ${:.2}", initial_cash);
        println!("Final Cash:          ${:.2}", self.balance);
        println!("Total Revenue:       ${:.2}", self.revenue);
        println!("Total Cost:          ${:.2}", self.total_cost);
        println!("Net Profit/Loss:     ${:.2}", self.revenue - self.total_cost);
        println!("Total Fees Paid:     ${:.2}", self.total_fees);
        println!("Net Cash Flow:       ${:.2}", self.cash_flow);
        println!("\nHoldings:");
        println!("Stock  | Shares   | Avg Cost    | Current Price | Unrealized P/L");
        println!("-------------------------------------------------------------");
        for (stock, (quantity, avg_cost)) in &self.holdings {
            let current_price = self.last_prices.get(stock).cloned().unwrap_or(*avg_cost);
            let unrealized_pl = (current_price - avg_cost) * (*quantity as f64);
            println!(
                "{:<6} | {:<8} | ${:<10.2} | ${:<12.2} | ${:<10.2}",
                stock, quantity, avg_cost, current_price, unrealized_pl
            );
        }
        println!("-------------------------------------------------------------\n");
    }
}
