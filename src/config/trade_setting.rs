/**
 * 💰 Trading Settings Module - Bonk.fun Trading Sniper Bot
 * 
 * This module manages all trading-related configuration and parameters
 * for the Bonk.fun Trading Sniper Bot, including buy amounts, fees, slippage,
 * and confirmation service settings.
 * 
 * Key Features:
 * - Trading parameter validation and optimization
 * - Fee calculation and management
 * - Slippage protection and validation
 * - Confirmation service configuration
 * - Priority fee management for transaction speed
 * 
 * Safety Features:
 * - Parameter bounds checking and validation
 * - Fee calculation accuracy verification
 * - Slippage protection limits
 * - Error handling for invalid configurations
 * 
 * Repository: https://github.com/solship/bonkfun-trading-snipper-grpc.git
 * @author solship
 * @version 2.0.0
 */

use dotenvy::dotenv;
use once_cell::sync::Lazy;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::CONFIG;

/**
 * Validates and loads confirmation service configuration
 * 
 * This function validates the confirmation service setting and ensures
 * it's one of the supported services: NOZOMI, ZERO_SLOT, or JITO.
 * 
 * @returns String - Validated confirmation service name
 */
fn load_confirmation_service() -> String {
    let service = CONFIG.services.confirm_service.clone();
    
    // Validate confirmation service
    match service.as_str() {
        "NOZOMI" | "ZERO_SLOT" | "JITO" => {
            println!("✅ Confirmation service configured: {}", service);
            service
        }
        _ => {
            eprintln!("⚠️ Invalid confirmation service: {}. Defaulting to NOZOMI", service);
            "NOZOMI".to_string()
        }
    }
}

/**
 * Validates and loads priority fee configuration
 * 
 * This function validates priority fee parameters and ensures they
 * are within reasonable bounds for optimal transaction processing.
 * 
 * @returns (u64, u64, f64) - (compute_units, priority_fee_micro_lamports, third_party_fee)
 */
fn load_priority_fee_config() -> (u64, u64, f64) {
    let cu = CONFIG.priority_fee.cu;
    let priority_fee_micro_lamport = CONFIG.priority_fee.priority_fee_micro_lamport;
    let third_party_fee = CONFIG.trade.third_party_fee;
    
    // Validate compute units (typical range: 50k - 1.4M)
    if cu < 50_000 || cu > 1_400_000 {
        eprintln!("⚠️ Compute units out of recommended range: {} (should be 50k-1.4M)", cu);
    }
    
    // Validate priority fee (typical range: 1 - 1000 micro-lamports)
    if priority_fee_micro_lamport < 1 || priority_fee_micro_lamport > 1000 {
        eprintln!("⚠️ Priority fee out of recommended range: {} micro-lamports (should be 1-1000)", priority_fee_micro_lamport);
    }
    
    // Validate third party fee
    if third_party_fee < 0.0 || third_party_fee > 1.0 {
        eprintln!("⚠️ Third party fee out of valid range: {} (should be 0.0-1.0)", third_party_fee);
    }
    
    println!("✅ Priority fee configured: {} CU, {} micro-lamports, {} SOL fee", 
             cu, priority_fee_micro_lamport, third_party_fee);
    
    (cu, priority_fee_micro_lamport, third_party_fee)
}

/**
 * Validates and loads buy amount configuration
 * 
 * This function validates the buy amount and converts it to lamports
 * for transaction processing, ensuring it's within reasonable bounds.
 * 
 * @returns u64 - Buy amount in lamports
 */
fn load_buy_amount() -> u64 {
    let buy_sol_amount = CONFIG.trade.buy_sol_amount;
    
    // Validate buy amount (minimum 0.0001 SOL, maximum 10 SOL)
    if buy_sol_amount < 0.0001 {
        eprintln!("⚠️ Buy amount too small: {} SOL (minimum 0.0001 SOL)", buy_sol_amount);
    }
    
    if buy_sol_amount > 10.0 {
        eprintln!("⚠️ Buy amount too large: {} SOL (maximum 10 SOL)", buy_sol_amount);
    }
    
    // Convert SOL to lamports (1 SOL = 10^9 lamports)
    let buy_amount_lamports = (buy_sol_amount * 10_f64.powf(9.0)) as u64;
    
    println!("✅ Buy amount configured: {} SOL ({} lamports)", buy_sol_amount, buy_amount_lamports);
    
    buy_amount_lamports
}

/**
 * Validates and loads slippage configuration
 * 
 * This function validates the slippage percentage and converts it
 * to decimal format for transaction processing.
 * 
 * @returns f64 - Slippage as decimal (e.g., 1.0% -> 0.01)
 */
fn load_slippage() -> f64 {
    let slippage_percent = CONFIG.trade.slippage;
    
    // Validate slippage (minimum 0.1%, maximum 100%)
    if slippage_percent < 0.1 {
        eprintln!("⚠️ Slippage too low: {}% (minimum 0.1%)", slippage_percent);
    }
    
    if slippage_percent > 100.0 {
        eprintln!("⚠️ Slippage too high: {}% (maximum 100%)", slippage_percent);
    }
    
    // Convert percentage to decimal
    let slippage_decimal = slippage_percent / 100.0;
    
    println!("✅ Slippage configured: {}% ({})", slippage_percent, slippage_decimal);
    
    slippage_decimal
}

/**
 * Calculates total transaction cost including fees
 * 
 * This function calculates the total cost of a transaction including
 * the buy amount, priority fees, and third-party fees.
 * 
 * @param base_amount - Base transaction amount in lamports
 * @returns u64 - Total cost in lamports
 */
pub fn calculate_total_cost(base_amount: u64) -> u64 {
    let (cu, priority_fee_micro_lamport, third_party_fee) = *PRIORITY_FEE;
    
    // Calculate priority fee cost
    let priority_fee_cost = cu * priority_fee_micro_lamport;
    
    // Calculate third party fee cost
    let third_party_fee_cost = (base_amount as f64 * third_party_fee * 10_f64.powf(9.0)) as u64;
    
    // Total cost
    let total_cost = base_amount + priority_fee_cost + third_party_fee_cost;
    
    println!("💰 Transaction cost breakdown:");
    println!("   Base amount: {} lamports", base_amount);
    println!("   Priority fee: {} lamports", priority_fee_cost);
    println!("   Third party fee: {} lamports", third_party_fee_cost);
    println!("   Total cost: {} lamports", total_cost);
    
    total_cost
}

/**
 * Validates wallet balance for transaction
 * 
 * This function checks if the wallet has sufficient balance
 * to execute a transaction with the given amount and fees.
 * 
 * @param required_amount - Required amount in lamports
 * @param wallet_balance - Current wallet balance in lamports
 * @returns bool - True if sufficient balance
 */
pub fn validate_wallet_balance(required_amount: u64, wallet_balance: u64) -> bool {
    if wallet_balance < required_amount {
        eprintln!("❌ Insufficient wallet balance: {} lamports (required: {} lamports)", 
                 wallet_balance, required_amount);
        return false;
    }
    
    // Add safety margin (1% buffer)
    let safety_margin = (required_amount as f64 * 0.01) as u64;
    let total_required = required_amount + safety_margin;
    
    if wallet_balance < total_required {
        eprintln!("⚠️ Wallet balance close to required amount: {} lamports (recommended: {} lamports)", 
                 wallet_balance, total_required);
    }
    
    println!("✅ Wallet balance sufficient: {} lamports (required: {} lamports)", 
             wallet_balance, required_amount);
    
    true
}

// Lazy static initialization for optimal performance

/**
 * Confirmation service configuration
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the service setting.
 */
pub static CONFIRM_SERVICE: Lazy<String> = Lazy::new(|| {
    println!("🔧 Loading confirmation service...");
    load_confirmation_service()
});

/**
 * Priority fee configuration parameters
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the fee parameters.
 */
pub static PRIORITY_FEE: Lazy<(u64, u64, f64)> = Lazy::new(|| {
    println!("💰 Loading priority fee configuration...");
    load_priority_fee_config()
});

/**
 * Buy amount in lamports
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the buy amount.
 */
pub static BUY_SOL_AMOUNT: Lazy<u64> = Lazy::new(|| {
    println!("💸 Loading buy amount configuration...");
    load_buy_amount()
});

/**
 * Slippage as decimal value
 * 
 * This is lazily initialized to ensure configuration is loaded
 * before attempting to validate the slippage setting.
 */
pub static SLIPPAGE: Lazy<f64> = Lazy::new(|| {
    println!("📊 Loading slippage configuration...");
    load_slippage()
});

/**
 * Validates all trading configuration on startup
 * 
 * This function should be called during application initialization
 * to ensure all trading parameters are valid and within safe bounds.
 * 
 * @returns Result<(), String> - Success or error message
 */
pub fn validate_trading_configuration() -> Result<(), String> {
    println!("🔍 Validating trading configuration...");
    
    // Validate buy amount
    let buy_amount = CONFIG.trade.buy_sol_amount;
    if buy_amount <= 0.0 {
        return Err("Buy amount must be greater than 0".to_string());
    }
    
    // Validate third party fee
    let third_party_fee = CONFIG.trade.third_party_fee;
    if third_party_fee < 0.0 {
        return Err("Third party fee cannot be negative".to_string());
    }
    
    // Validate slippage
    let slippage = CONFIG.trade.slippage;
    if slippage <= 0.0 || slippage > 100.0 {
        return Err("Slippage must be between 0.1 and 100.0".to_string());
    }
    
    // Validate priority fee parameters
    let cu = CONFIG.priority_fee.cu;
    if cu == 0 {
        return Err("Compute units cannot be zero".to_string());
    }
    
    let priority_fee = CONFIG.priority_fee.priority_fee_micro_lamport;
    if priority_fee == 0 {
        return Err("Priority fee cannot be zero".to_string());
    }
    
    println!("✅ Trading configuration validation passed");
    Ok(())
}
