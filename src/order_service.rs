use log::{info, error};
use tonic::{Request, Response, Status};
use crate::order_repository::OrderRepository;
use crate::order_service::proto::{OrderRequest, OrderResponse, OrderListResponse, Empty, OrderEntityResponse, OrderEntityLineItems, DeleteOrderRequest, DeleteOrderResponse};
use itertools::Itertools;
use uuid::Uuid;
use crate::models::OrderLineItems;
use crate::order_service::proto::order_server::Order;
use prost_types::Timestamp;

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
                id: None,
                sku_code: item.sku_code.to_owned(),
                price: item.price,
                quantity: item.quantity,
            }
        }).collect();
        let order = crate::models::Order {
            id: None,
            order_number: Uuid::new_v4().to_string(),
            created_at: None,
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

    async fn get_order_list(&self, _: Request<Empty>) -> Result<Response<OrderListResponse>, Status> {
        info!("Received order get_order_list request");

        let orders = self.repository.get_order_list().await
            .map_err(|err| {
                error!("Failed to get_order_list {:?}", err);
                Status::internal("could not get_order_list")
            })?;

        let order_responses: Vec<OrderEntityResponse> = orders.into_iter().map(|order| {
            OrderEntityResponse {
                order_id: order.id.unwrap_or_default(),
                order_number: order.order_number,
                created_at: order.created_at.map(|t| Timestamp {
                    seconds: t.timestamp(),
                    nanos: t.timestamp_subsec_nanos() as i32
                }),
                items: order.items.into_iter().map(|item| {
                    OrderEntityLineItems {
                        order_line_item_id: item.id.unwrap_or_default(),
                        sku_code: item.sku_code,
                        price: item.price,
                        quantity: item.quantity,
                    }
                }).collect(),
            }
        }).collect();

        info!("get_order_list successfully returned orders={}", order_responses.len());
        Ok(Response::new(OrderListResponse {
            orders: order_responses
        }))
    }

    async fn delete_order(&self, request: Request<DeleteOrderRequest>) -> Result<Response<DeleteOrderResponse>, Status> {
        let order_request = request.get_ref();
        info!("Received delete_order request order_id={}", order_request.order_id);

        let is_deleted = self.repository.delete(order_request.order_id).await.map_err(|err| {
            error!("Failed to delete_order {:?}", err);
            Status::internal(format!("could not delete order_id={}", order_request.order_id))
        })?;

        let mut deleted_msg = "deleted";
        if !is_deleted {
            deleted_msg = "not deleted";
        }
        info!("order_id={} is {}", order_request.order_id, deleted_msg);
        Ok(Response::new(DeleteOrderResponse {
            is_deleted
        }))
    }
}