use log::{info, error};
use tonic::{Request, Response, Status};
use crate::order_repository::OrderRepository;
use crate::order_service::proto::{OrderRequest, OrderResponse};
use itertools::Itertools;
use uuid::Uuid;
use crate::models::OrderLineItems;
use crate::order_service::proto::order_server::Order;

pub mod proto {
    tonic::include_proto!("order"); // product is a package in product.proto file
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("order_descriptor");
}

pub struct OrderService {
    repository: OrderRepository
}

impl OrderService {
    pub fn new(order_repository: OrderRepository) -> Self {
        OrderService {
            repository: order_repository
        }
    }
}

#[tonic::async_trait]
impl Order for OrderService {
    async fn place(&self, request: Request<OrderRequest>) -> Result<Response<OrderResponse>, Status> {
        let order_request = request.get_ref();
        let sku_codes = order_request.items.iter()
            .map(|x| &x.sku_code).join(",");
        info!("Received order place request sku_codes: {}", sku_codes);

        let items: Vec<OrderLineItems> = order_request.items.iter().map(|item| {
            OrderLineItems {
                sku_code: item.sku_code.to_owned(),
                price: item.price,
                quantity: item.quantity,
            }
        }).collect();
        let order = crate::models::Order {
            order_number: Uuid::new_v4().to_string(),
            items,
        };
        let order_response = self.repository.save(&order).await
            .map_err(|err| {
                error!("Failed to save order {:?}", err);
                Status::internal("could not save order")
            })?;
        info!("Order placed successfully id: {}", order_response);
        Ok(Response::new(OrderResponse { order_number: order_response }))
    }
}