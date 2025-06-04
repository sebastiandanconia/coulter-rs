//! REST (axum) presentation layer for the "BTC constants" endpoint.
//!
//! This module is transport-specific: it turns HTTP requests into calls to the
//! shared business-logic functions ([`crate::get_sec256k1_prime`] /
//! [`crate::get_genesis_hash`]) and turns the results back into HTTP responses.
//! It is reused by `src/main.rs` (the both-at-once binary) and by
//! `examples/rest_only.rs`, so the REST behaviour can't drift between them.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use prost::Message;

use crate::proto::btc_constants::{BtcConstantRequest, BtcConstantResponse};
use crate::{get_genesis_hash, get_sec256k1_prime};

/// Max request body size accepted by the REST handler (1 MiB).
pub const BUFFER_SZ: usize = 1024 * 1024;

/// JSON request body for the REST endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct JsonRequestPayload {
    #[serde(rename = "transaction-id")]
    pub transaction_id: String,
    #[serde(rename = "client-id")]
    pub client_id: String,
}

/// JSON response body for the REST endpoint.
#[derive(Debug, serde::Serialize)]
pub struct JsonResponsePayload {
    #[serde(rename = "transaction-id")]
    pub transaction_id: String,
    #[serde(rename = "sec256k1-prime")]
    pub sec256k1_prime: String,
    #[serde(rename = "btc-genesis-hash")]
    pub genesis_hash: String,
}

/// Shared state made available to the REST (axum) handlers via
/// `axum::extract::State`.
///
/// Designed to be shared across many tasks; wrap any interior mutability in a
/// `tokio::sync::RwLock` / `Mutex` as appropriate.
pub struct AppState {
    /// Placeholder for a database handle.
    pub db: String,
    /// Placeholder for secrets / credentials.
    pub etherscan_api_key: Option<Arc<tokio::sync::RwLock<String>>>,
}

impl AppState {
    /// Construct an `AppState` wrapped in an `Arc`, ready to hand to [`build_router`].
    pub fn new(db: &str) -> Arc<Self> {
        Arc::new(AppState {
            db: db.to_string(),
            etherscan_api_key: None,
        })
    }
}

/// Build the axum [`axum::Router`] for the REST API.
///
/// The single handler at `POST /api/btc-constants/v1` dispatches on the
/// request `Content-Type`:
///   * `application/json`        -> JSON request/response
///   * `application/x-protobuf`  -> raw protobuf request/response
///                                  (protobuf-over-HTTP; this is *not* gRPC,
///                                   which uses HTTP/2 + protobuf framing)
pub fn build_router(state: Arc<AppState>) -> axum::Router {
    axum::Router::new()
        .route("/api/btc-constants/v1", axum::routing::post(btc_constants_handler))
        .with_state(state)
}

/// Axum handler for `POST /api/btc-constants/v1`.
///
/// Both branches call the *same* shared library functions
/// (`get_sec256k1_prime`, `get_genesis_hash`) that the gRPC service uses, so the
/// REST and gRPC responses are always identical for the same inputs.
#[axum::debug_handler]
async fn btc_constants_handler(
    headers: HeaderMap,
    State(_state): State<Arc<AppState>>,
    request: Request,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .ok_or((StatusCode::BAD_REQUEST, "Missing Content-Type".to_string()))?
        .to_str()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid Content-Type: {e}")))?;

    match content_type {
        "application/json" => {
            // Read and parse the JSON body.
            let request_bytes = axum::body::to_bytes(request.into_body(), BUFFER_SZ)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let request_payload: JsonRequestPayload = serde_json::from_slice(&request_bytes)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {e}")))?;

            if request_payload.transaction_id.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "transaction-id is required".to_string(),
                ));
            }

            // Build the response from the shared business logic.
            let transaction_id = &request_payload.transaction_id;
            let client_id = &request_payload.client_id;
            let prime = hex::encode(get_sec256k1_prime(transaction_id, client_id));
            let genesis_hash = hex::encode(get_genesis_hash(transaction_id, client_id));

            let response_body = serde_json::to_vec(&JsonResponsePayload {
                transaction_id: transaction_id.to_string(),
                sec256k1_prime: prime,
                genesis_hash,
            })
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize JSON data: {e}"),
                )
            })?;

            let mut response_headers = HeaderMap::new();
            response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

            Ok((StatusCode::OK, response_headers, response_body))
        }

        "application/x-protobuf" => {
            // Read and decode the protobuf body. (See also `prost::length_delimiter_*`
            // helpers if you need length-delimited framing.)
            let request_bytes = axum::body::to_bytes(request.into_body(), BUFFER_SZ)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let request_payload = BtcConstantRequest::decode(&request_bytes[..])
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid Protobuf: {e}")))?;

            if request_payload.transaction_id.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "transaction-id is required".to_string(),
                ));
            }

            // Build the response from the shared business logic, using the same
            // `BtcConstantResponse` message type the gRPC service returns.
            let transaction_id = &request_payload.transaction_id;
            let client_id = &request_payload.client_id;
            let pb_response = BtcConstantResponse {
                transaction_id: transaction_id.to_string(),
                sec256k1_prime: get_sec256k1_prime(transaction_id, client_id).to_vec(),
                btc_genesis_hash: get_genesis_hash(transaction_id, client_id).to_vec(),
            };

            let mut response_buf = Vec::new();
            pb_response
                .encode(&mut response_buf)
                .expect("Failed to serialize Protobuf");

            let mut response_headers = HeaderMap::new();
            response_headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/x-protobuf"),
            );

            Ok((StatusCode::OK, response_headers, response_buf))
        }

        _ => Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported Content-Type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn check_send_sync() {
        assert_send_sync::<AppState>();
    }
}
