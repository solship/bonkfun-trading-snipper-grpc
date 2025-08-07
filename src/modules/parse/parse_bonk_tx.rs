/**
 * 🔍 Transaction Parsing Module - Bonk.fun Trading Sniper Bot
 * 
 * This module handles the parsing and analysis of Solana transactions
 * to detect Bonk.fun trading opportunities and extract relevant data.
 * 
 * Key Features:
 * - Transaction data extraction from gRPC updates
 * - Bonk.fun instruction parsing and validation
 * - Account key resolution and validation
 * - Trading opportunity detection and analysis
 * - Comprehensive error handling and logging
 * 
 * Repository: https://github.com/solship/bonkfun-trading-snipper-grpc.git
 * @author solship
 * @version 2.0.0
 */

use crate::{
    BONK_BUY_IN_DISC, BONK_INIT_DISC, BonkBuy, BonkBuyParam, BonkfunMIntInfo, MoonBuy,
    MoonBuyParamWrapper, MoonshotMintInfo, PumpfunBuy, PumpfunBuyParam, PumpfunMintInfo,
    RAYDIUM_LAUNCHPAD_PROGRAM_ID, parse_bonk_initialize_params,
};
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;
use yellowstone_grpc_proto::{
    geyser::{SubscribeUpdate, subscribe_update::UpdateOneof},
    prelude::CompiledInstruction,
};

/**
 * Extracts transaction data from gRPC update
 * 
 * This function parses the gRPC transaction update and extracts:
 * - Account keys (including loaded addresses)
 * - Compiled instructions
 * - Transaction signature/ID
 * 
 * @param update - gRPC transaction update
 * @returns Option<(Vec<Pubkey>, Vec<CompiledInstruction>, String)> - Parsed data or None
 */
pub fn extract_transaction_data(
    update: &SubscribeUpdate,
) -> Option<(Vec<Pubkey>, Vec<CompiledInstruction>, String)> {
    // Extract transaction update from enum
    let transaction_update = match &update.update_oneof {
        Some(UpdateOneof::Transaction(tx_update)) => tx_update,
        _ => {
            // Skip non-transaction updates
            return None;
        }
    };

    // Safely extract nested transaction data with error handling
    let tx_info = transaction_update.transaction.as_ref()?;
    let transaction = tx_info.transaction.as_ref()?;
    let meta = tx_info.meta.as_ref()?;
    let tx_msg = transaction.message.as_ref()?;

    // Parse account keys from transaction message
    let mut account_keys: Vec<Pubkey> = parse_account_keys(&tx_msg.account_keys)?;

    // Append loaded writable addresses from transaction metadata
    account_keys.extend(parse_loaded_addresses(&meta.loaded_writable_addresses)?);

    // Append loaded readonly addresses from transaction metadata
    account_keys.extend(parse_loaded_addresses(&meta.loaded_readonly_addresses)?);

    // Extract compiled instructions
    let ixs: Vec<CompiledInstruction> = tx_msg.instructions.clone();

    // Parse transaction signature/ID
    let signature = &tx_info.signature;
    let tx_id = bs58::encode(signature).into_string();

    Some((account_keys, ixs, tx_id))
}

/**
 * Parses account keys from raw bytes
 * 
 * @param account_keys_raw - Raw account key bytes
 * @returns Option<Vec<Pubkey>> - Parsed public keys or None
 */
fn parse_account_keys(account_keys_raw: &[Vec<u8>]) -> Option<Vec<Pubkey>> {
    let mut account_keys = Vec::new();
    
    for key_bytes in account_keys_raw {
        match Pubkey::try_from(key_bytes.as_slice()) {
            Ok(pubkey) => account_keys.push(pubkey),
            Err(e) => {
                eprintln!("⚠️ Failed to parse account key: {}", e);
                // Continue parsing other keys
            }
        }
    }
    
    if account_keys.is_empty() {
        None
    } else {
        Some(account_keys)
    }
}

/**
 * Parses loaded addresses from transaction metadata
 * 
 * @param loaded_addresses - Raw loaded address bytes
 * @returns Option<Vec<Pubkey>> - Parsed public keys or None
 */
fn parse_loaded_addresses(loaded_addresses: &[Vec<u8>]) -> Option<Vec<Pubkey>> {
    let mut addresses = Vec::new();
    
    for address_bytes in loaded_addresses {
        match Pubkey::try_from(address_bytes.as_slice()) {
            Ok(pubkey) => addresses.push(pubkey),
            Err(e) => {
                eprintln!("⚠️ Failed to parse loaded address: {}", e);
                // Continue parsing other addresses
            }
        }
    }
    
    Some(addresses)
}

/**
 * Analyzes transaction for trading opportunities
 * 
 * This function processes compiled instructions to detect:
 * - Bonk.fun token initialization events
 * - Bonk.fun buy transactions
 * - Trading parameters and account structures
 * 
 * @param ixs - Compiled instructions from transaction
 * @param account_keys - Account keys involved in transaction
 * @returns (Option<BonkfunMIntInfo>, Option<BonkBuy>, Option<BonkBuyParam>) - Trading data
 */
pub fn trade_info(
    ixs: Vec<CompiledInstruction>,
    account_keys: Vec<Pubkey>,
) -> (
    Option<BonkfunMIntInfo>,
    Option<BonkBuy>,
    Option<BonkBuyParam>,
) {
    let mut bonk_mint: Option<BonkfunMIntInfo> = None;
    let mut bonk_buy: Option<BonkBuy> = None;
    let mut bonk_buy_param: Option<BonkBuyParam> = None;

    // Process each instruction in the transaction
    for (ix_index, ix) in ixs.iter().enumerate() {
        // Validate instruction data length
        if ix.data.len() < 8 {
            eprintln!("⚠️ Instruction {} has insufficient data length", ix_index);
            continue;
        }

        // Check if this is a Bonk.fun program instruction
        let program_id = match account_keys.get(ix.program_id_index as usize) {
            Some(id) => id,
            None => {
                eprintln!("⚠️ Invalid program ID index: {}", ix.program_id_index);
                continue;
            }
        };

        // Process Bonk.fun initialization instruction
        if ix.data.starts_with(&BONK_INIT_DISC) && (*program_id == RAYDIUM_LAUNCHPAD_PROGRAM_ID) {
            bonk_mint = parse_bonk_initialization_instruction(ix, ix_index);
        }
        // Process Bonk.fun buy instruction
        else if ix.data.starts_with(&BONK_BUY_IN_DISC) && (*program_id == RAYDIUM_LAUNCHPAD_PROGRAM_ID) {
            let (buy, param) = parse_bonk_buy_instruction(ix, &account_keys, ix_index);
            bonk_buy = buy;
            bonk_buy_param = param;
        }
    }

    (bonk_mint, bonk_buy, bonk_buy_param)
}

/**
 * Parses Bonk.fun initialization instruction
 * 
 * @param ix - Compiled instruction
 * @param ix_index - Instruction index for logging
 * @returns Option<BonkfunMIntInfo> - Parsed mint info or None
 */
fn parse_bonk_initialization_instruction(
    ix: &CompiledInstruction,
    ix_index: usize,
) -> Option<BonkfunMIntInfo> {
    match parse_bonk_initialize_params(&ix.data) {
        Ok(mint_data) => {
            println!("🎯 Bonk.fun initialization detected in instruction {}", ix_index);
            Some(mint_data)
        }
        Err(e) => {
            eprintln!("❌ Failed to parse Bonk.fun initialization in instruction {}: {}", ix_index, e);
            None
        }
    }
}

/**
 * Parses Bonk.fun buy instruction
 * 
 * @param ix - Compiled instruction
 * @param account_keys - Account keys involved in transaction
 * @param ix_index - Instruction index for logging
 * @returns (Option<BonkBuy>, Option<BonkBuyParam>) - Parsed buy data
 */
fn parse_bonk_buy_instruction(
    ix: &CompiledInstruction,
    account_keys: &[Pubkey],
    ix_index: usize,
) -> (Option<BonkBuy>, Option<BonkBuyParam>) {
    // Validate account count for Bonk.fun buy instruction
    if ix.accounts.len() < 15 {
        eprintln!("❌ Invalid Bonk.fun buy account layout in instruction {}: expected 15, got {}", 
                 ix_index, ix.accounts.len());
        return (None, None);
    }

    // Extract account keys with bounds checking
    let bonk_fun_buy = match extract_bonk_buy_accounts(ix, account_keys) {
        Ok(buy) => buy,
        Err(e) => {
            eprintln!("❌ Failed to extract Bonk.fun buy accounts in instruction {}: {}", ix_index, e);
            return (None, None);
        }
    };

    // Parse buy parameters
    let bonk_buy_param = match BonkBuyParam::deserialize(&mut &ix.data[8..]) {
        Ok(param) => {
            println!("🎯 Bonk.fun buy instruction detected in instruction {}", ix_index);
            Some(param)
        }
        Err(e) => {
            eprintln!("❌ Failed to parse Bonk.fun buy parameters in instruction {}: {}", ix_index, e);
            None
        }
    };

    (Some(bonk_fun_buy), bonk_buy_param)
}

/**
 * Extracts account keys for Bonk.fun buy instruction
 * 
 * @param ix - Compiled instruction
 * @param account_keys - All account keys in transaction
 * @returns Result<BonkBuy, String> - Parsed buy structure or error
 */
fn extract_bonk_buy_accounts(
    ix: &CompiledInstruction,
    account_keys: &[Pubkey],
) -> Result<BonkBuy, String> {
    // Validate account indices
    for (i, &account_index) in ix.accounts.iter().enumerate() {
        if account_index as usize >= account_keys.len() {
            return Err(format!("Account index {} out of bounds (max: {})", account_index, account_keys.len() - 1));
        }
    }

    Ok(BonkBuy {
        payer: account_keys[ix.accounts[0] as usize],
        authority: account_keys[ix.accounts[1] as usize],
        global_config: account_keys[ix.accounts[2] as usize],
        platform_config: account_keys[ix.accounts[3] as usize],
        pool_state: account_keys[ix.accounts[4] as usize],
        user_base_token: account_keys[ix.accounts[5] as usize],
        user_quote_token: account_keys[ix.accounts[6] as usize],
        base_vault: account_keys[ix.accounts[7] as usize],
        quote_vault: account_keys[ix.accounts[8] as usize],
        base_token_mint: account_keys[ix.accounts[9] as usize],
        quote_token_mint: account_keys[ix.accounts[10] as usize],
        base_token_program: account_keys[ix.accounts[11] as usize],
        quote_token_program: account_keys[ix.accounts[12] as usize],
        event_authority: account_keys[ix.accounts[13] as usize],
        program: account_keys[ix.accounts[14] as usize],
    })
}
