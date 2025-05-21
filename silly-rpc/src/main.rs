use std::sync::{Arc};
use axum::RequestExt;
use hyper::{server, HeaderMap};
use axum::http::{HeaderName, HeaderValue};

use prost::Message;
use proto::btc_constants::{self, BtcConstantRequest, BtcConstantResponse};
use tokio::net::TcpListener;
use tonic::transport::Server as TonicServer;
use std::net::SocketAddr;

use axum::http::{header, StatusCode, request};
use axum::response::{IntoResponse, Response};
use axum::extract::Request;

use crate::proto::btc_constants::btc_constant_service_server::BtcConstantServiceServer;
use crate::grpc_service::BtcConstantServiceImpl;

use silly_rpc::{
    get_sec256k1_prime,
    get_genesis_hash,
};

mod proto;
mod grpc_service;

const BUFFER_SZ: usize = 1024*1024;

#[derive(Debug, serde::Deserialize)]
struct JsonRequestPayload {
    #[serde(rename = "transaction-id")]
    transaction_id: String,
    #[serde(rename = "client-id")]
    client_id: String,
}

#[derive(Debug, serde::Serialize)]
struct JsonResponsePayload {
    #[serde(rename = "transaction-id")]
    transaction_id: String,
    #[serde(rename = "sec256k1-prime")]
    sec256k1_prime: String,
    #[serde(rename = "btc-genesis-hash")]
    genesis_hash: String,
}

// Shared state (e.g., database connection)
struct AppState {
    // This struct is designed to be shared by many threads. Manage Read/Write mutex if necessary.
    db: String, // Placeholder for database
    etherscan_api_key: Option<tokio::sync::RwLock<String>>, // Placeholder for API keys, credentials, etc.
}


#[tokio::main]
async fn main() {
    ////////////////////////////////////////////////////////////////////////////////
    // To serve REST and gRPC simulataneously, see:
    // https://rust-on-nails.com/docs/api/grpc/
    // https://github.com/jvdwrf/axum-tonic/blob/main/src/rest_grpc.rs
    // https://github.com/maxnachlinger/rest-grpc-reflection-multiplex/blob/main/src/multiplex_service.rs
    ////////////////////////////////////////////////////////////////////////////////

    ////////////////////////////////////////////////////////////////////////////////
    // Use this code to create a gRPC server
    ////////////////////////////////////////////////////////////////////////////////
    let grpc_service = BtcConstantServiceServer::new(BtcConstantServiceImpl::new(
        Arc::new("gRPC Database Placeholder".to_string()))
    );
    
    // Convert tonic server to axum Router
    let grpc_router = TonicServer::builder()
        .add_service(grpc_service)
        .serve("0.0.0.0:4000".parse().unwrap())
        .await
        .unwrap();
    ////////////////////////////////////////////////////////////////////////////////


    ////////////////////////////////////////////////////////////////////////////////
    // Use this code to create a HTTP 1.1 server
    ////////////////////////////////////////////////////////////////////////////////
    let app_state = AppState{
        db: "Database connection placeholder".to_string(),
        etherscan_api_key: None,
    };
    // Define our axum router.
    // The compiler throws a fit if this router is created without later using it in axum::serve().
    let app = axum::Router::new()
        // .route("/api/noop", axum::routing::post(|| async { "Did nothing successfully" }))
        .route("/api/illegal-number/v1", axum::routing::post(illegal_handler))
        .with_state(app_state.into());

    // Create a TCP listener
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    // Serve the app using axum::serve
    axum::serve(listener, app.into_make_service()).await.unwrap();
    ////////////////////////////////////////////////////////////////////////////////
}


/**
 Debugging handler type errors

For a function to be used as a handler it must implement the Handler trait. axum provides blanket implementations for functions that:

- Are async fns.
- Take no more than 16 arguments that all implement Send.
    - All except the last argument implement FromRequestParts.
    - The last argument implements FromRequest.
- Returns something that implements IntoResponse.
- If a closure is used it must implement Clone + Send and be 'static.
- Returns a future that is Send. The most common way to accidentally make a future !Send is to hold a !Send type across an await.
 */

#[axum::debug_handler]
async fn illegal_handler(
    headers: HeaderMap,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    request: Request,
) -> Result<impl IntoResponse, (StatusCode, String)> {

    let content_type = headers.get("Content-Type")
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing Content-Type".to_string()))?
        .to_str()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid Content-Type: {}", e)))?;

    match content_type {
        "application/json" => {
            // Extract payload from request body
            let request_bytes = axum::body::to_bytes(request.into_body(), BUFFER_SZ)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let request_payload: JsonRequestPayload = serde_json::from_slice(&request_bytes)
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

            // Validate input
            if request_payload.transaction_id.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "transaction-id is required".to_string()));
            }

            // Generate response
            let transaction_id = &request_payload.transaction_id;
            let client_id = &request_payload.client_id;
            let prime = hex::encode(get_sec256k1_prime(transaction_id, client_id));
            let genesis_hash = hex::encode(get_genesis_hash(transaction_id, client_id));
            let response_json = axum::extract::Json(JsonResponsePayload {
                transaction_id: transaction_id.to_string(),
                sec256k1_prime: prime,
                genesis_hash: genesis_hash,
            });

            // Serialize the inner data to JSON bytes
            let response_body = serde_json::to_vec(&response_json.0)
                .map_err(|e| { (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialize JSON data: {}", e)) })?;

            let mut response_headers = HeaderMap::new();
            response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

            // Use StatusCode::OK as the default status
            Ok((StatusCode::OK, response_headers, response_body))
        }
        "application/x-protobuf" => {
            // See also:
            // prost::length_delimiter_length()
            // prost::encode_length_delimiter()
            // prost::decode_length_delimiter()

            // Extract payload from request body
            let request_bytes = axum::body::to_bytes(request.into_body(), BUFFER_SZ)
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            let request_payload: BtcConstantRequest = BtcConstantRequest::decode(&request_bytes[..])
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid Protobuf: {}", e)))?;

            // Validate input
            if request_payload.transaction_id.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "transaction-id is required".to_string()));
            }

            // Generate response
            let transaction_id = &request_payload.transaction_id;
            let client_id = &request_payload.client_id;
            let prime = get_sec256k1_prime(transaction_id, client_id);
            let genesis_hash = get_genesis_hash(transaction_id, client_id);

            let mut response_buf = Vec::new();

            let pb_response = BtcConstantResponse {
                transaction_id: transaction_id.to_string(),
                sec256k1_prime: prime.to_vec(),
                btc_genesis_hash: genesis_hash.to_vec(),
            };
            pb_response.encode(& mut response_buf).expect("Failed to serialize Protobuf");

            let mut response_headers = HeaderMap::new();
            response_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/x-protobuf"));

            Ok((StatusCode::OK, response_headers, response_buf))
        }
        _ => Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported Content-Type".to_string()))
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn handler_implements_handler() {
        // fn assert_handler<H: axum::handler::Handler<(), Arc<AppState>>>(_h: H) {}
        // assert_handler(illegal_handler);
    }

    #[test]
    fn check_send_sync() {
        assert_send_sync::<AppState>();
        //check_app_state();
    }

}