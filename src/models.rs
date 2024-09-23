
pub struct Order {
    pub order_number: String,
    pub items: Vec<OrderLineItems>
}

pub struct OrderLineItems {
    pub sku_code: String,
    pub price: i64,
    pub quantity: i64,
}