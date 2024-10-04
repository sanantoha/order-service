use sqlx::types::chrono::{DateTime, Utc};

pub struct Order {
    pub id: Option<i64>,
    pub order_number: String,
    pub created_at: Option<DateTime<Utc>>,
    pub items: Vec<OrderLineItems>
}

pub struct OrderLineItems {
    pub id: Option<i64>,
    pub sku_code: String,
    pub price: i64,
    pub quantity: i64,
}