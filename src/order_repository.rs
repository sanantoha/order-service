use sqlx::{MySql, Pool, Transaction};
use sqlx::types::chrono::Utc;
use crate::error::Error;
use crate::models::Order;

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
}