//! silly-rpc binary: serve gRPC **and** REST at the same time, from one process.
//!
//! gRPC (HTTP/2 + protobuf framing) listens on `0.0.0.0:50051`.
//! REST (HTTP/1.1, JSON or protobuf-over-HTTP) listens on `0.0.0.0:3000`.
//!
//! Everything that is actually *served* — the protobuf messages, the gRPC
//! service impl, and the REST router/handler — lives in the library crate
//! (`silly_rpc`) so it can be shared. This file is purely *orchestration*: it
//! wires those pieces up to two servers and drives them concurrently with
//! `tokio::select!` (whichever finishes first cancels the other).
//!
//! # Related examples
//!
//! * `cargo run --example grpc_only` — gRPC only, in `examples/grpc_only.rs`.
//! * `cargo run --example rest_only` — REST only, in `examples/rest_only.rs`.
//!
//! # Serving both on a single port
//!
//! The two-port approach below is what most production systems do: it is
//! simple, robust, and lets each protocol use the port/transport that suits it.
//! Serving both on a *single* port by multiplexing on `Content-Type` (gRPC =>
//! HTTP/2, REST => HTTP/1.1) is possible but more complex and brittle. If you
//! need that, these references are good starting points:
//!   * https://rust-on-nails.com/docs/api/grpc/
//!   * https://github.com/jvdwrf/axum-tonic
//!   * https://github.com/maxnachlinger/rest-grpc-reflection-multiplex

use std::future::IntoFuture;
use std::net::SocketAddr;
use std::sync::Arc;

use silly_rpc::grpc_service::BtcConstantServiceImpl;
use silly_rpc::proto::btc_constants::btc_constant_service_server::BtcConstantServiceServer;
use silly_rpc::rest;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // ---- Shared building blocks (defined once; see the library crate). ----

    // State handed to the gRPC service implementation.
    let grpc_state = Arc::new("gRPC database placeholder".to_string());

    // State handed to the REST handlers.
    let rest_state = rest::AppState::new("REST database placeholder");

    // The gRPC service (tonic). `BtcConstantServiceImpl` implements the
    // `BtcConstantService` trait generated from the .proto file.
    let grpc_service = BtcConstantServiceServer::new(BtcConstantServiceImpl::new(grpc_state));

    // The REST router (axum). See `rest::build_router` for the Content-Type
    // dispatch (application/json vs application/x-protobuf).
    let rest_router = rest::build_router(rest_state);

    // ---- Bind two ports and serve both concurrently. ----
    let grpc_addr: SocketAddr = "0.0.0.0:50051".parse().unwrap();
    let rest_addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    // `tonic`'s `.serve(addr)` returns a `Future` directly.
    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc_service)
        .serve(grpc_addr);

    // `axum::serve(...)` returns a `Serve` that implements `IntoFuture` (not
    // `Future`), so convert it before using it in `tokio::select!`.
    let rest_listener = TcpListener::bind(rest_addr).await.unwrap();
    let rest_server = axum::serve(rest_listener, rest_router.into_make_service()).into_future();

    println!("gRPC listening on http://{grpc_addr}");
    println!("REST listening on http://{rest_addr}");

    // Whichever server finishes (or errors) first cancels the other.
    tokio::select! {
        result = grpc_server => match result {
            Ok(()) => println!("gRPC server stopped"),
            Err(e) => eprintln!("gRPC server error: {e}"),
        },
        result = rest_server => match result {
            Ok(()) => println!("REST server stopped"),
            Err(e) => eprintln!("REST server error: {e}"),
        },
    }

    // =========================================================================
    // OPTIONAL: gRPC-Web support (browser clients).
    //
    // `tonic-web` lets a browser speak the grpc-web protocol to the same gRPC
    // service without an external proxy. Apply the `GrpcWebLayer` to the tonic
    // server above and enable HTTP/1 on it. This is orthogonal to REST; it only
    // affects the gRPC server.
    //
    //     tonic::transport::Server::builder()
    //         .accept_http1(true)
    //         .layer(tonic_web::GrpcWebLayer::new())
    //         .add_service(grpc_service)
    //         .serve(grpc_addr)
    //         .await
    //         .unwrap();
    // =========================================================================
}
