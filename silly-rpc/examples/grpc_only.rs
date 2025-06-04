//! gRPC-only example: serve `BtcConstantService` over gRPC on `0.0.0.0:50051`.
//!
//! Run with:
//! ```sh
//! cargo run --example grpc_only
//! ```
//! then test with:
//! ```sh
//! python3 grpc_test_utility.py --transaction_id tx123 --client_id cli456
//! ```
//!
//! Compare with `src/main.rs` (gRPC + REST together) and
//! `examples/rest_only.rs` (REST only). All three reuse the same service
//! implementation from `silly_rpc::grpc_service`.

use std::net::SocketAddr;
use std::sync::Arc;

use silly_rpc::grpc_service::BtcConstantServiceImpl;
use silly_rpc::proto::btc_constants::btc_constant_service_server::BtcConstantServiceServer;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();

    // The same service impl used by `src/main.rs`; it calls the shared
    // business-logic functions in `silly_rpc`.
    let db = Arc::new("gRPC database placeholder".to_string());
    let service = BtcConstantServiceServer::new(BtcConstantServiceImpl::new(db));

    println!("gRPC listening on http://{addr}");

    // Optional gRPC-Web (browser) support: add `.accept_http1(true)` and
    // `.layer(tonic_web::GrpcWebLayer::new())` before `.add_service` so that
    // grpc-web clients can reach this server without an external proxy.
    tonic::transport::Server::builder()
        .add_service(service)
        .serve(addr)
        .await
        .unwrap();
}
