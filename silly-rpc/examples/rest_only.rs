//! REST-only example: serve the axum HTTP API on `0.0.0.0:3000`.
//!
//! The single endpoint `POST /api/btc-constants/v1` dispatches on the request
//! `Content-Type`:
//!   * `application/json`        -> JSON request/response
//!   * `application/x-protobuf`  -> raw protobuf request/response
//!                                  (protobuf-over-HTTP; this is *not* gRPC,
//!                                   which uses HTTP/2 + protobuf framing)
//!
//! Run with:
//! ```sh
//! cargo run --example rest_only
//! ```
//! then test with:
//! ```sh
//! # JSON
//! curl -s -X POST http://localhost:3000/api/btc-constants/v1 \
//!   -H 'Content-Type: application/json' \
//!   -d '{"transaction-id":"tx123","client-id":"cli456"}'
//! # protobuf-over-HTTP
//! python3 protobuf_test_utility.py --transaction_id tx123 --client_id cli456
//! ```
//!
//! Compare with `src/main.rs` (gRPC + REST together) and
//! `examples/grpc_only.rs` (gRPC only). All three reuse the same REST router
//! and handler from `silly_rpc::rest`, which in turn call the same shared
//! business-logic functions as the gRPC service.

use std::net::SocketAddr;

use silly_rpc::rest;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    // The same router/handler used by `src/main.rs`; it calls the shared
    // business-logic functions in `silly_rpc`.
    let state = rest::AppState::new("REST database placeholder");
    let router = rest::build_router(state);

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("REST listening on http://{addr}");

    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
