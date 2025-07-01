use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeMessage {
    pub stock_id: String,
    pub current_price: f64,
    pub action_type: String, 
    pub quantity: u32,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TradeResponse {
    pub stock_id: String,
    pub decision: String, 
    pub quantity: u32,
    pub price: f64,
    pub timestamp: String,
}
