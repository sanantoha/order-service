use std::collections::HashMap;
use sqlx::{MySql, Pool, Transaction, Row};
use sqlx::types::chrono::{DateTime, Utc};
use crate::error::Error;
use crate::models::{Order, OrderLineItems};

pub struct OrderRepository {
    pool: Pool<MySql>
}

impl OrderRepository {

    pub fn new(pool: Pool<MySql>) -> OrderRepository {
        OrderRepository {
            pool
        }
    }

    pub async fn save(&self, order: &Order) -> Result<String, Error> {
        let mut transaction: Transaction<'_, MySql> = self.pool.begin().await?;
        let order_query = r#"
            INSERT INTO `order-service`.`t_orders` (order_number, created_at)
            VALUES (?, ?);
        "#;
        let created_time = Utc::now().naive_utc();

        let order_insert_result = sqlx::query(order_query)
            .bind(&order.order_number)
            .bind(created_time)
            .execute(&mut *transaction)
            .await?;

        let inserted_order_id = order_insert_result.last_insert_id();

        let order_item_query = r#"
            INSERT INTO `order-service`.`t_order_line_items` (sku_code, price, quantity, order_id)
            VALUES (?, ?, ?, ?);
        "#;

        for item in &order.items {
            sqlx::query(order_item_query)
                .bind(&item.sku_code)
                .bind(&item.price)
                .bind(item.quantity)
                .bind(inserted_order_id)
                .execute(&mut *transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(order.order_number.to_owned())
    }

    pub async fn get_order_list(&self) -> Result<Vec<Order>, Error> {
        let order_query = r#"
            SELECT
                o.id AS order_id, o.order_number, o.created_at, oi.id AS order_item_id, oi.sku_code, oi.price, oi.quantity
            FROM `order-service`.`t_orders` AS o
                INNER JOIN `order-service`.`t_order_line_items` AS oi ON o.id = oi.order_id
        "#;

        let rows = sqlx::query(order_query).fetch_all(&self.pool).await?;

        let mut cache = HashMap::new();

        for row in rows {
            let order_id: i64 = row.try_get("order_id")?;
            let order_number: String = row.try_get("order_number")?;
            let created_at: Option<DateTime<Utc>> = row.try_get("created_at")?;
            let sku_code: String = row.try_get("sku_code")?;
            let price: i64 = row.try_get("price")?;
            let quantity: i64 = row.try_get("quantity")?;
            let order_item_id: i64 = row.try_get("order_item_id")?;

            let order = cache.entry(order_id).or_insert_with(|| {
                Order {
                    id: Some(order_id),
                    order_number,
                    created_at: created_at,
                    items: vec![],
                }
            });

            let order_item = OrderLineItems {
                id: Some(order_item_id),
                sku_code,
                price,
                quantity,
            };

            order.items.push(order_item);
        }
        let res: Vec<Order> = cache.into_values().collect();

        Ok(res)
    }
}