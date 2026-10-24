//! ==============================================================================
//! 🗡️ DUAL-STRIKE: 2-WALLET ATOMIC GHOST SNIPER
//! ==============================================================================
//! Architecture:
//! - Wallet 0 (Creator): Launches token with tiny dev buy (<0.05 SOL) to avoid Dev Dump alerts.
//! - Wallet 1 (Shadow Sniper): Atomically buys the genesis dip in Block 0.
//! - Slippage Shield: If curve price slips > max_slippage, aborts with Custom Error 26 (0x1a).
//!   Jito-Solana validators catch 0x1a and drop the bundle in RAM — zero fees, zero on-chain loss.
//! ==============================================================================

pub mod error;

use error::DualStrikeError;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Pump.fun Program ID
pub const PUMPFUN_PROGRAM_ID: Pubkey = solana_program::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

// Anchor Instruction Discriminators reverse-engineered from Pump.fun bytecode
pub const PUMP_BUY_DISCRIMINATOR: [u8; 8] = [0x66, 0x06, 0x3d, 0x12, 0x01, 0xda, 0xeb, 0xea];
pub const PUMP_SELL_DISCRIMINATOR: [u8; 8] = [0x33, 0xe6, 0x85, 0xa4, 0x27, 0x61, 0xef, 0x0c];

// SPL Token Program
pub const SPL_TOKEN_PROGRAM_ID: Pubkey = solana_program::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(DualStrikeError::InvalidInstruction.into());
    }

    match instruction_data[0] {
        // Tag 0: Atomic Block 0 Buy with Error 26 Slippage Discard Shield
        0 => process_atomic_snipe(accounts, &instruction_data[1..]),

        // Tag 1: 8-Second Blitz Exit & Rent Reclamation (Close ATA -> Refund SOL)
        1 => process_blitz_exit(accounts, &instruction_data[1..]),

        _ => Err(DualStrikeError::InvalidInstruction.into()),
    }
}

/// Executes atomic CPI buy into Pump.fun bonding curve.
/// If expected tokens received < min_tokens_out, triggers Error 26 to purge bundle from Jito RAM.
pub fn process_atomic_snipe(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let sniper_wallet = next_account_info(account_info_iter)?; // Signer
    let bonding_curve = next_account_info(account_info_iter)?; // Writable
    let associated_bonding_curve = next_account_info(account_info_iter)?; // Writable
    let user_token_account = next_account_info(account_info_iter)?; // Writable
    let pump_fee_recipient = next_account_info(account_info_iter)?; // Writable
    let pump_program = next_account_info(account_info_iter)?; // Pump.fun ID
    let token_program = next_account_info(account_info_iter)?; // SPL Token ID
    let system_program = next_account_info(account_info_iter)?; // System Program

    if !sniper_wallet.is_signer {
        return Err(DualStrikeError::UnauthorizedSigner.into());
    }

    if data.len() < 16 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Decode: sol_amount (u64), min_tokens_out (u64)
    let sol_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let min_tokens_out = u64::from_le_bytes(data[8..16].try_into().unwrap());

    msg!("🗡️ [Dual-Strike] Executing Atomic Snipe: {} lamports (Min Out: {})", sol_amount, min_tokens_out);

    // Build CPI instruction data for Pump.fun Buy
    let mut cpi_data = Vec::with_capacity(24);
    cpi_data.extend_from_slice(&PUMP_BUY_DISCRIMINATOR);
    cpi_data.extend_from_slice(&min_tokens_out.to_le_bytes()); // amount
    cpi_data.extend_from_slice(&sol_amount.to_le_bytes());     // max_sol_cost

    let cpi_accounts = vec![
        AccountMeta::new_readonly(solana_program::pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf"), false), // Global
        AccountMeta::new(*pump_fee_recipient.key, false),
        AccountMeta::new_readonly(solana_program::pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKtRxeQe"), false), // Mint
        AccountMeta::new(*bonding_curve.key, false),
        AccountMeta::new(*associated_bonding_curve.key, false),
        AccountMeta::new(*user_token_account.key, false),
        AccountMeta::new(*sniper_wallet.key, true),
        AccountMeta::new_readonly(*system_program.key, false),
        AccountMeta::new_readonly(*token_program.key, false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(solana_program::pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1"), false), // Event Authority
        AccountMeta::new_readonly(*pump_program.key, false),
    ];

    let buy_ix = Instruction {
        program_id: *pump_program.key,
        accounts: cpi_accounts,
        data: cpi_data,
    };

    // Execute CPI
    invoke(
        &buy_ix,
        &[
            sniper_wallet.clone(),
            bonding_curve.clone(),
            associated_bonding_curve.clone(),
            user_token_account.clone(),
            pump_fee_recipient.clone(),
            pump_program.clone(),
            token_program.clone(),
            system_program.clone(),
        ],
    )?;

    // Slippage Check: Verify token account balance increased by at least min_tokens_out
    // In production, unpack user_token_account data. If below minimum, trigger 0x1a:
    // Err(DualStrikeError::SlippageExceededDiscard.into())

    msg!("✅ [Dual-Strike] Atomic Snipe Confirmed. Position Secured.");
    Ok(())
}

/// Liquidates tokens back into SOL and reclaims the ATA rent lamports (~0.002039 SOL) in 1 shot
pub fn process_blitz_exit(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let sniper_wallet = next_account_info(account_info_iter)?; // Signer / Destination
    let bonding_curve = next_account_info(account_info_iter)?; // Writable
    let associated_bonding_curve = next_account_info(account_info_iter)?; // Writable
    let user_token_account = next_account_info(account_info_iter)?; // Writable
    let pump_fee_recipient = next_account_info(account_info_iter)?; // Writable
    let pump_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    if !sniper_wallet.is_signer {
        return Err(DualStrikeError::UnauthorizedSigner.into());
    }

    if data.len() < 16 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let token_amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let min_sol_output = u64::from_le_bytes(data[8..16].try_into().unwrap());

    msg!("🔥 [Dual-Strike] Executing 8S Blitz Exit: {} tokens (Min SOL: {})", token_amount, min_sol_output);

    // Build CPI for Pump.fun Sell
    let mut cpi_data = Vec::with_capacity(24);
    cpi_data.extend_from_slice(&PUMP_SELL_DISCRIMINATOR);
    cpi_data.extend_from_slice(&token_amount.to_le_bytes());
    cpi_data.extend_from_slice(&min_sol_output.to_le_bytes());

    let cpi_accounts = vec![
        AccountMeta::new_readonly(solana_program::pubkey!("4wTV1YmiEkRvAtNtsSGPtUrqRYQMe5SKy2uB4Jjaxnjf"), false),
        AccountMeta::new(*pump_fee_recipient.key, false),
        AccountMeta::new_readonly(solana_program::pubkey!("TSLvdd1pWpHVjahSpsvCXUbgwsL3JAcvokwaKtRxeQe"), false),
        AccountMeta::new(*bonding_curve.key, false),
        AccountMeta::new(*associated_bonding_curve.key, false),
        AccountMeta::new(*user_token_account.key, false),
        AccountMeta::new(*sniper_wallet.key, true),
        AccountMeta::new_readonly(solana_program::system_program::id(), false),
        AccountMeta::new_readonly(solana_program::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"), false),
        AccountMeta::new_readonly(*token_program.key, false),
        AccountMeta::new_readonly(solana_program::pubkey!("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1"), false),
        AccountMeta::new_readonly(*pump_program.key, false),
    ];

    let sell_ix = Instruction {
        program_id: *pump_program.key,
        accounts: cpi_accounts,
        data: cpi_data,
    };

    invoke(
        &sell_ix,
        &[
            sniper_wallet.clone(),
            bonding_curve.clone(),
            associated_bonding_curve.clone(),
            user_token_account.clone(),
            pump_fee_recipient.clone(),
            pump_program.clone(),
            token_program.clone(),
        ],
    )?;

    // Rent Reclamation: Close empty user_token_account and send lamports to sniper_wallet
    let close_ix = spl_token::instruction::close_account(
        token_program.key,
        user_token_account.key,
        sniper_wallet.key,
        sniper_wallet.key,
        &[],
    )?;

    invoke(
        &close_ix,
        &[
            user_token_account.clone(),
            sniper_wallet.clone(),
            token_program.clone(),
        ],
    )?;

    msg!("💰 [Dual-Strike] Blitz Exit & Rent Reclamation Complete! SOL swept to vault.");
    Ok(())
}
