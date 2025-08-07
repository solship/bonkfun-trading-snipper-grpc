/**
 * 🚀 Bonk.fun Trading Sniper Bot - Main Entry Point
 * 
 * This is the primary entry point for the Bonk.fun Trading Sniper Bot.
 * It orchestrates the entire process of monitoring Solana transactions
 * via gRPC streams and executing automated trading strategies.
 * 
 * Key Features:
 * - Real-time transaction monitoring via Helius Laserstream gRPC
 * - Automated token sniping on Bonk.fun launches
 * - Multi-service transaction confirmation (Nozomi, Zero Slot, Jito)
 * - Configurable trading parameters and filters
 * - High-performance Rust implementation
 * 
 * Repository: https://github.com/solship/bonkfun-trading-snipper-grpc.git
 * @author solship
 * @version 2.0.0
 */

use bonk_sniper_rust::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

/// Main application entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Bonk.fun Trading Sniper Bot v2.0.0...");
    println!("📦 Repository: https://github.com/solship/bonkfun-trading-snipper-grpc.git");
    println!("👨‍💻 Author: solship");
    
    // Initialize external services and global state
    initialize_services().await?;
    
    // Start background tasks for optimal performance
    start_background_tasks().await?;
    
    // Setup and start gRPC transaction monitoring
    start_transaction_monitoring().await?;
    
    Ok(())
}

/**
 * Initializes all external services and clients
 * 
 * This function sets up connections to various services:
 * - Nozomi confirmation service
 * - Zero Slot confirmation service  
 * - Jito bundle service
 * 
 * @returns Result<(), Box<dyn std::error::Error>> - Success or error
 */
async fn initialize_services() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing external services...");
    
    // Initialize confirmation services
    init_nozomi().await;
    init_zslot().await;
    init_jito().await;
    
    println!("✅ External services initialized successfully");
    Ok(())
}

/**
 * Starts background tasks for optimal performance
 * 
 * Background tasks include:
 * - Blockhash management for transaction signing
 * - Health monitoring and metrics collection
 * - Connection keep-alive management
 * 
 * @returns Result<(), Box<dyn std::error::Error>> - Success or error
 */
async fn start_background_tasks() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting background tasks...");
    
    // Start blockhash handler loop in background for optimal performance
    tokio::spawn(async {
        println!("📡 Blockhash handler started");
        loop {
            match recent_blockhash_handler(RPC_CLIENT.clone()).await {
                Ok(_) => {
                    // Successfully updated blockhash
                }
                Err(e) => {
                    eprintln!("❌ Blockhash handler error: {}", e);
                    // Continue running despite errors
                }
            }
            
            // Small delay to prevent excessive RPC calls
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
    
    println!("✅ Background tasks started successfully");
    Ok(())
}

/**
 * Sets up and starts gRPC transaction monitoring
 * 
 * This function:
 * 1. Establishes gRPC connection to Helius Laserstream
 * 2. Configures transaction filters for Bonk.fun programs
 * 3. Starts processing transaction updates
 * 4. Handles connection errors and reconnection
 * 
 * @returns Result<(), Box<dyn std::error::Error>> - Success or error
 */
async fn start_transaction_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Setting up gRPC transaction monitoring...");
    
    // Setup gRPC client with error handling
    let mut grpc_client = match setup_client_grpc(GRPC_ENDPOINT.to_string(), GRPC_TOKEN.to_string()).await {
        Ok(client) => {
            println!("✅ gRPC client connected successfully");
            client
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to gRPC: {}", e);
            return Err(e);
        }
    };

    // Setup subscription channel
    let (subscribe_tx, subscribe_rx) = match grpc_client.subscribe().await {
        Ok(channel) => {
            println!("✅ gRPC subscription channel established");
            channel
        }
        Err(e) => {
            eprintln!("❌ Failed to create subscription channel: {}", e);
            return Err(Box::new(e));
        }
    };

    // Configure transaction filters for Bonk.fun programs
    let subscribe_filter = create_transaction_filter();
    
    // Send subscription request with error handling
    match send_subscription_request_grpc(subscribe_tx, subscribe_filter).await {
        Ok(_) => {
            println!("✅ Transaction filter subscription sent successfully");
        }
        Err(e) => {
            eprintln!("❌ Failed to send subscription request: {}", e);
            return Err(e);
        }
    }

    // Start processing transaction updates with comprehensive error handling
    println!("🎯 Starting transaction processing loop...");
    match process_updates_grpc(subscribe_rx).await {
        Ok(_) => {
            println!("✅ Transaction processing completed successfully");
        }
        Err(e) => {
            eprintln!("❌ Error processing transaction updates: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/**
 * Creates transaction filter for Bonk.fun programs
 * 
 * This function configures which programs to monitor:
 * - Moonshot program for token launches
 * - Pump.fun program for pump tokens
 * - Raydium launchpad for liquidity events
 * 
 * @returns SubscribeRequestFilterTransactions - Configured filter
 */
fn create_transaction_filter() -> SubscribeRequestFilterTransactions {
    println!("🔍 Configuring transaction filters...");
    
    SubscribeRequestFilterTransactions {
        account_include: vec![
            MOONSHOT_PROGRAM_ID.to_string(),
            PUMP_FUN_PROGRAM_ID.to_string(),
            RAYDIUM_LAUNCHPAD_PROGRAM_ID.to_string(),
        ],
        account_exclude: vec![],
        account_required: vec![],
        vote: Some(false),
        failed: Some(false),
        signature: None,
    }
}
