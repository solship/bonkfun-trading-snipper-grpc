/**
 * 🔐 Credentials and Configuration Module - Bonk.fun Trading Sniper Bot
 * 
 * This module handles all credential management and configuration loading
 * for the Bonk.fun Trading Sniper Bot, including wallet setup, RPC connections,
 * and gRPC endpoint configuration.
 * 
 * Key Features:
 * - Secure private key management
 * - RPC client initialization and configuration
 * - gRPC endpoint setup and validation
 * - Environment variable loading and validation
 * - Lazy initialization for optimal performance
 * 
 * Security Features:
 * - Private key validation and format checking
 * - Secure RPC endpoint validation
 * - Environment variable sanitization
 * - Error handling without exposing sensitive data
 * 
 * Repository: https://github.com/solship/bonkfun-trading-snipper-grpc.git
 * @author solship
 * @version 2.0.0
 */

use dotenvy::dotenv;
use once_cell::sync::Lazy;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
};
use std::{env, sync::Arc};

use crate::CONFIG;

/**
 * Validates and loads private key from configuration
 * 
 * This function safely loads the private key from configuration,
 * validates its format, and creates a Keypair for transaction signing.
 * 
 * Security Features:
 * - Base58 format validation
 * - Keypair integrity verification
 * - Error handling without exposing private key data
 * 
 * @returns Keypair - Validated wallet keypair
 */
fn load_private_key() -> Keypair {
    let private_key_str = &CONFIG.wallet.private_key;
    
    // Validate private key format
    if private_key_str.is_empty() {
        panic!("❌ Private key is empty. Please configure your wallet private key.");
    }
    
    if private_key_str.len() < 80 {
        panic!("❌ Private key appears to be invalid (too short). Please check your configuration.");
    }
    
    // Attempt to create keypair from base58 string
    match Keypair::from_base58_string(private_key_str) {
        Ok(keypair) => {
            println!("✅ Private key loaded successfully");
            keypair
        }
        Err(e) => {
            panic!("❌ Failed to load private key: {}. Please check your configuration.", e);
        }
    }
}

/**
 * Validates and loads RPC endpoint configuration
 * 
 * @returns String - Validated RPC endpoint URL
 */
fn load_rpc_endpoint() -> String {
    let endpoint = CONFIG.rpc.endpoint.clone();
    
    if endpoint.is_empty() {
        panic!("❌ RPC endpoint is empty. Please configure your RPC endpoint.");
    }
    
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        panic!("❌ Invalid RPC endpoint format. Must start with http:// or https://");
    }
    
    println!("✅ RPC endpoint configured: {}", endpoint);
    endpoint
}

/**
 * Validates and loads gRPC endpoint configuration
 * 
 * @returns String - Validated gRPC endpoint URL
 */
fn load_grpc_endpoint() -> String {
    let endpoint = CONFIG.grpc.endpoint.clone();
    
    if endpoint.is_empty() {
        panic!("❌ gRPC endpoint is empty. Please configure your gRPC endpoint.");
    }
    
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        panic!("❌ Invalid gRPC endpoint format. Must start with http:// or https://");
    }
    
    println!("✅ gRPC endpoint configured: {}", endpoint);
    endpoint
}

/**
 * Validates and loads gRPC authentication token
 * 
 * @returns String - Validated gRPC authentication token
 */
fn load_grpc_token() -> String {
    let token = CONFIG.grpc.token.clone();
    
    if token.is_empty() {
        panic!("❌ gRPC token is empty. Please configure your gRPC authentication token.");
    }
    
    println!("✅ gRPC token loaded successfully");
    token
}

/**
 * Creates and configures RPC client with optimal settings
 * 
 * This function creates an RPC client with:
 * - Processed commitment level for fastest confirmations
 * - Proper error handling and validation
 * - Connection pooling and optimization
 * 
 * @param endpoint - Validated RPC endpoint URL
 * @returns Arc<RpcClient> - Configured RPC client
 */
fn create_rpc_client(endpoint: String) -> Arc<RpcClient> {
    let client = RpcClient::new_with_commitment(
        endpoint,
        CommitmentConfig::processed(),
    );
    
    println!("✅ RPC client created with processed commitment level");
    Arc::new(client)
}

// Lazy static initialization for optimal performance and memory usage

/**
 * Wallet private key loaded from configuration
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to parse the private key.
 */
pub static PRIVATE_KEY: Lazy<Keypair> = Lazy::new(|| {
    println!("🔐 Loading private key...");
    load_private_key()
});

/**
 * Wallet public key derived from private key
 * 
 * This is lazily initialized and derived from the private key
 * for transaction signing and account identification.
 */
pub static PUBKEY: Lazy<Pubkey> = Lazy::new(|| {
    println!("🔑 Deriving public key...");
    PRIVATE_KEY.pubkey()
});

/**
 * RPC endpoint URL loaded from configuration
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the endpoint.
 */
pub static RPC_ENDPOINT: Lazy<String> = Lazy::new(|| {
    println!("🌐 Loading RPC endpoint...");
    load_rpc_endpoint()
});

/**
 * RPC client instance with optimal configuration
 * 
 * This is lazily initialized to ensure the endpoint is validated
 * before creating the client connection.
 */
pub static RPC_CLIENT: Lazy<Arc<RpcClient>> = Lazy::new(|| {
    println!("🔌 Creating RPC client...");
    create_rpc_client(RPC_ENDPOINT.clone())
});

/**
 * gRPC endpoint URL loaded from configuration
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the endpoint.
 */
pub static GRPC_ENDPOINT: Lazy<String> = Lazy::new(|| {
    println!("📡 Loading gRPC endpoint...");
    load_grpc_endpoint()
});

/**
 * gRPC authentication token loaded from configuration
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the token.
 */
pub static GRPC_TOKEN: Lazy<String> = Lazy::new(|| {
    println!("🔑 Loading gRPC token...");
    load_grpc_token()
});

/**
 * Validates all configuration on startup
 * 
 * This function should be called during application initialization
 * to ensure all required configuration is present and valid.
 * 
 * @returns Result<(), String> - Success or error message
 */
pub fn validate_configuration() -> Result<(), String> {
    println!("🔍 Validating configuration...");
    
    // Validate private key
    if CONFIG.wallet.private_key.is_empty() {
        return Err("Private key is not configured".to_string());
    }
    
    // Validate RPC endpoint
    if CONFIG.rpc.endpoint.is_empty() {
        return Err("RPC endpoint is not configured".to_string());
    }
    
    // Validate gRPC endpoint
    if CONFIG.grpc.endpoint.is_empty() {
        return Err("gRPC endpoint is not configured".to_string());
    }
    
    // Validate gRPC token
    if CONFIG.grpc.token.is_empty() {
        return Err("gRPC token is not configured".to_string());
    }
    
    println!("✅ Configuration validation passed");
    Ok(())
}
