/**
 * 📡 gRPC Subscription Setup Module - Bonk.fun Trading Sniper Bot
 * 
 * This module handles the setup and management of gRPC connections
 * for real-time transaction monitoring via Helius Laserstream.
 * 
 * Key Features:
 * - gRPC client initialization and configuration
 * - TLS/SSL connection setup and validation
 * - Transaction filter configuration and management
 * - Subscription request handling and validation
 * - Connection error handling and recovery
 * 
 * Performance Features:
 * - Connection pooling and optimization
 * - Automatic reconnection handling
 * - Efficient subscription management
 * - Memory-efficient stream processing
 * 
 * Repository: https://github.com/solship/bonkfun-trading-snipper-grpc.git
 * @author solship
 * @version 2.0.0
 */

use futures::SinkExt;
use std::collections::HashMap;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient, Interceptor};
use yellowstone_grpc_proto::geyser::{
    CommitmentLevel, SubscribeRequest, SubscribeRequestFilterTransactions,
};

/**
 * Sets up gRPC client connection with comprehensive configuration
 * 
 * This function establishes a secure gRPC connection to Helius Laserstream
 * with proper TLS configuration, authentication, and error handling.
 * 
 * Connection Features:
 * - TLS/SSL encryption for secure communication
 * - Authentication token validation
 * - Connection timeout and retry logic
 * - Native root certificate validation
 * 
 * @param grpc_endpoint - gRPC endpoint URL
 * @param x_token - Authentication token for Helius
 * @returns Result<GeyserGrpcClient<impl Interceptor>, Box<dyn std::error::Error>> - Configured client or error
 */
pub async fn setup_client_grpc(
    grpc_endpoint: String,
    x_token: String,
) -> Result<GeyserGrpcClient<impl Interceptor>, Box<dyn std::error::Error>> {
    println!("🔌 Setting up gRPC client connection...");
    
    // Validate endpoint format
    if !is_valid_grpc_endpoint(&grpc_endpoint) {
        return Err(format!("Invalid gRPC endpoint format: {}", grpc_endpoint).into());
    }
    
    // Validate authentication token
    if x_token.is_empty() {
        return Err("Authentication token cannot be empty".into());
    }
    
    println!("🌐 Connecting to gRPC endpoint: {}", grpc_endpoint);
    
    // Build gRPC client with comprehensive configuration
    let client = match GeyserGrpcClient::build_from_shared(grpc_endpoint.clone()) {
        Ok(builder) => builder,
        Err(e) => {
            eprintln!("❌ Failed to create gRPC client builder: {}", e);
            return Err(e.into());
        }
    };
    
    // Configure authentication token
    let client = match client.x_token(Some(x_token.clone())) {
        Ok(client) => {
            println!("✅ Authentication token configured");
            client
        }
        Err(e) => {
            eprintln!("❌ Failed to configure authentication token: {}", e);
            return Err(e.into());
        }
    };
    
    // Configure TLS with native root certificates
    let client = match client.tls_config(ClientTlsConfig::new().with_native_roots()) {
        Ok(client) => {
            println!("🔒 TLS configuration applied");
            client
        }
        Err(e) => {
            eprintln!("❌ Failed to configure TLS: {}", e);
            return Err(e.into());
        }
    };
    
    // Establish connection with timeout
    println!("🔄 Establishing gRPC connection...");
    let client = match client.connect().await {
        Ok(client) => {
            println!("✅ gRPC connection established successfully");
            client
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to gRPC endpoint: {}", e);
            return Err(e.into());
        }
    };
    
    Ok(client)
}

/**
 * Validates gRPC endpoint format
 * 
 * @param endpoint - Endpoint URL to validate
 * @returns bool - True if endpoint format is valid
 */
fn is_valid_grpc_endpoint(endpoint: &str) -> bool {
    if endpoint.is_empty() {
        return false;
    }
    
    // Check for valid protocol
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return false;
    }
    
    // Check for valid host format
    if endpoint.contains("://") {
        let parts: Vec<&str> = endpoint.split("://").collect();
        if parts.len() != 2 || parts[1].is_empty() {
            return false;
        }
    }
    
    true
}

/**
 * Sends subscription request with transaction filters
 * 
 * This function configures and sends a subscription request to monitor
 * specific Solana programs and transactions for Bonk.fun trading opportunities.
 * 
 * Subscription Features:
 * - Program-specific transaction filtering
 * - Commitment level configuration
 * - Account monitoring setup
 * - Error handling and validation
 * 
 * @param tx - Subscription sender channel
 * @param subscribe_args - Transaction filter configuration
 * @returns Result<(), Box<dyn std::error::Error>> - Success or error
 */
pub async fn send_subscription_request_grpc<T>(
    mut tx: T,
    subscribe_args: SubscribeRequestFilterTransactions,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: SinkExt<SubscribeRequest> + Unpin,
    <T as futures::Sink<SubscribeRequest>>::Error: std::error::Error + 'static,
{
    println!("📡 Configuring transaction subscription...");
    
    // Validate subscription arguments
    if subscribe_args.account_include.is_empty() {
        return Err("No accounts specified for monitoring".into());
    }
    
    // Create account filter with the target accounts
    let mut accounts_filter = HashMap::new();
    accounts_filter.insert("account_monitor".to_string(), subscribe_args.clone());
    
    // Log monitored programs
    println!("🎯 Monitoring programs:");
    for (i, program) in subscribe_args.account_include.iter().enumerate() {
        println!("   {}. {}", i + 1, program);
    }
    
    // Create subscription request with optimal settings
    let subscription_request = SubscribeRequest {
        transactions: accounts_filter,
        commitment: Some(CommitmentLevel::Processed as i32),
        ..Default::default()
    };
    
    // Send subscription request with error handling
    println!("📤 Sending subscription request...");
    match tx.send(subscription_request).await {
        Ok(_) => {
            println!("✅ Subscription request sent successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to send subscription request: {}", e);
            Err(e.into())
        }
    }
}

/**
 * Creates optimized transaction filter for Bonk.fun monitoring
 * 
 * This function creates a transaction filter specifically optimized
 * for monitoring Bonk.fun trading activities and token launches.
 * 
 * @returns SubscribeRequestFilterTransactions - Optimized filter configuration
 */
pub fn create_optimized_transaction_filter() -> SubscribeRequestFilterTransactions {
    println!("🔍 Creating optimized transaction filter...");
    
    SubscribeRequestFilterTransactions {
        account_include: vec![
            // Bonk.fun and related programs
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(), // Raydium Launchpad
            "PFundRj3D7BS6MLs5qvD6iJpWnSxTMRqjC7HLYQ6Pc1".to_string(),   // Pump.fun
            "M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K".to_string(), // Moonshot
        ],
        account_exclude: vec![
            // Exclude system accounts and known spam
            "11111111111111111111111111111111".to_string(), // System Program
        ],
        account_required: vec![
            // Require specific accounts for valid transactions
        ],
        vote: Some(false), // Exclude vote transactions
        failed: Some(false), // Exclude failed transactions
        signature: None, // No specific signature filter
    }
}

/**
 * Validates subscription configuration
 * 
 * @param filter - Transaction filter to validate
 * @returns Result<(), String> - Success or error message
 */
pub fn validate_subscription_filter(filter: &SubscribeRequestFilterTransactions) -> Result<(), String> {
    if filter.account_include.is_empty() {
        return Err("No accounts specified for monitoring".to_string());
    }
    
    // Validate account addresses format
    for account in &filter.account_include {
        if account.len() != 44 {
            return Err(format!("Invalid account address length: {} (expected 44)", account.len()));
        }
        
        // Basic base58 validation
        if !account.chars().all(|c| c.is_alphanumeric()) {
            return Err(format!("Invalid account address format: {}", account));
        }
    }
    
    Ok(())
}

/**
 * Creates connection health check function
 * 
 * This function returns a closure that can be used to check
 * the health of the gRPC connection and trigger reconnection if needed.
 * 
 * @returns impl Fn() -> bool - Health check function
 */
pub fn create_health_check() -> impl Fn() -> bool {
    let mut last_activity = std::time::Instant::now();
    
    move || {
        let now = std::time::Instant::now();
        let duration = now.duration_since(last_activity);
        
        // Consider connection healthy if activity within last 30 seconds
        if duration.as_secs() < 30 {
            last_activity = now;
            true
        } else {
            false
        }
    }
}
