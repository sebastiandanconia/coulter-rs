//! silly-rpc: a worked example of exposing the same business logic over both
//! gRPC and REST from one binary.
//!
//! This library crate holds the *transport-agnostic* pieces that every serving
//! mode shares:
//!
//!   * [`proto`] — the protobuf messages & gRPC service trait generated from
//!     `src/proto/btc_constants.proto` by `build.rs` (tonic-build).
//!   * [`grpc_service`] — the gRPC service implementation
//!     ([`grpc_service::BtcConstantServiceImpl`]).
//!   * [`rest`] — the REST (axum) presentation layer: the [`rest::AppState`],
//!     the [`rest::build_router`] helper, and the request handler.
//!   * [`get_sec256k1_prime`] / [`get_genesis_hash`] — the "business logic"
//!     both transports call, so gRPC and REST always return identical results.
//!
//! The crate is built as both a library and a binary. The binary
//! (`src/main.rs`) serves gRPC **and** REST at the same time. Two cargo
//! examples show the single-transport variants:
//!
//!   * `cargo run --example grpc_only` — gRPC only (`examples/grpc_only.rs`)
//!   * `cargo run --example rest_only` — REST only (`examples/rest_only.rs`)
//!
//! (Serving both on a *single* port by multiplexing on `Content-Type` is
//! possible but more complex; it is left as a future exercise — see the
//! references at the bottom of `src/main.rs`.)

pub mod grpc_service;
pub mod proto;
pub mod rest;

const SECP256K1_PRIME: [u8; 32] = [
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    0xFF, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFC, 0x2F,
];

const BITCOIN_GENESIS_HASH: [u8; 32] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xD6, 0x68,
    0x9C, 0x08, 0x5A, 0xE1, 0x65, 0x83, 0x1E, 0x93,
    0x4F, 0xF7, 0x63, 0xAE, 0x46, 0xA2, 0xA6, 0xC1,
    0x72, 0xB3, 0xF1, 0xB6, 0x0A, 0x8C, 0xE2, 0x6F,
];

// These functions stand in for whatever reusable functions the frontends call.
// They can also be methods on, or associated with, shared state. Both the gRPC
// service and the REST handler call them, so the two transports always agree.
pub fn get_genesis_hash(_transaction_id: &str, _client_id: &str) -> &'static [u8] {
    &BITCOIN_GENESIS_HASH
}

pub fn get_sec256k1_prime(_transaction_id: &str, _client_id: &str) -> &'static [u8] {
    &SECP256K1_PRIME
}
