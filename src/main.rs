use std::env;
use log::info;

mod error;
mod order_service;
mod order_repository;
mod models;

use crate::error::Error;
use crate::order_repository::OrderRepository;

use tonic::transport::Server;
use crate::order_service::OrderService;
use crate::order_service::proto::order_server::OrderServer;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Pool, MySql};

const ORDER_SERVICE_PORT_NAME: &str = "ORDER_PORT";

const ORDER_SERVICE_DATABASE_URL: &str = "ORDER_SERVICE_DATABASE_URL";

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", std::env::var("RUST_LOG").unwrap_or("info".to_owned()));
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let port = env::var(ORDER_SERVICE_PORT_NAME)
        .map_err(|e| Error::Var { input: ORDER_SERVICE_PORT_NAME, source: e })?;

    let database_url = env::var(ORDER_SERVICE_DATABASE_URL)
        .map_err(|e| Error::Var { input: ORDER_SERVICE_DATABASE_URL, source: e })?;

    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(1) // Set the max number of connections
        .connect(&database_url).await?;

    let order_repository = OrderRepository::new(pool);
    let order_service = OrderService::new(order_repository);

    let addr = format!("[::1]:{}", port).parse()?;
    info!("Listening on port {:?}", addr);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(order_service::proto::FILE_DESCRIPTOR_SET)
        .build_v1().map_err(|e| Error::Generic(e.to_string()))?;

    Server::builder()
        .add_service(reflection_service)
        .add_service(OrderServer::new(order_service))
        .serve(addr)
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;

    Ok(())
}
