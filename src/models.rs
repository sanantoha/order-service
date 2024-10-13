use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Ord, Eq, PartialEq, PartialOrd)]
pub struct Order {
    pub id: Option<i64>,
    pub order_number: String,
    pub created_at: Option<DateTime<Utc>>,
    pub items: Vec<OrderLineItems>
}

#[derive(Debug, Ord, Eq, PartialOrd, PartialEq)]
pub struct OrderLineItems {
    pub id: Option<i64>,
    pub sku_code: String,
    pub price: i64,
    pub quantity: i64,
}