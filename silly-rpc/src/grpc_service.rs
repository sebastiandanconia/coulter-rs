use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::proto::btc_constants::{BtcConstantRequest, BtcConstantResponse};

use crate::proto::btc_constants::btc_constant_service_server::BtcConstantService;

use silly_rpc::{
    get_sec256k1_prime,
    get_genesis_hash,
};

// Define the service implementation struct
pub struct BtcConstantServiceImpl {
    // Optional shared state (e.g., a database connection)
    db: Arc<String>, // Placeholder for a real database connection
}

impl BtcConstantServiceImpl {
    // Constructor to initialize the service with shared state
    pub fn new(db: Arc<String>) -> Self {
        Self { db }
    }
}

// Implement the generated MyService trait
#[tonic::async_trait]
impl BtcConstantService for BtcConstantServiceImpl {
    async fn get_btc_constants(
        &self,
        request: Request<BtcConstantRequest>,
    ) -> Result<Response<BtcConstantResponse>, Status> {
        // Extract the request data

        let msg = request.into_inner();

        // Example: Incorporate shared state (here, just a string)
        // let message = format!("Hello, {}! (from {})", transaction_id, self.db);
        
        let sec256k1_prime = get_sec256k1_prime(&msg.transaction_id, &msg.client_id);
        let btc_genesis_hash = get_genesis_hash(&msg.transaction_id, &msg.client_id);

        // Create the response
        let response = BtcConstantResponse {
            transaction_id: msg.transaction_id,
            sec256k1_prime: sec256k1_prime.to_vec(),
            btc_genesis_hash: btc_genesis_hash.to_vec()
        };

        // Return the response wrapped in tonic's Response type
        Ok(Response::new(response))
    }
}