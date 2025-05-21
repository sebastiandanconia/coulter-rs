// src/proto/mod.rs

// Declare the rpc_service submodule
pub mod btc_constants {
    include!(concat!(env!("OUT_DIR"), "/btc_constants.rs"));
}

// Add more submodules here if needed
// For example:
// pub mod another_service;
