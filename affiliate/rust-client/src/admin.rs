use crate::utils::{default_keypair, get_or_create_ata, simulate_transaction};
use anyhow::Result;
use solana_program::sysvar;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use std::str::FromStr;

pub fn init_partner(
    program_client: &anchor_client::Program,
    vault: Pubkey,
    partner: String,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::InitPartner {
            partner,
            vault,
            partner_token,
            admin: program_client.payer(),
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
            token_program: spl_token::id(),
        })
        .args(affiliate::instruction::InitPartner {});

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}

pub fn update_fee_ratio(
    program_client: &anchor_client::Program,
    vault: Pubkey,
    partner: String,
    fee_ratio: u64,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner)?;

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::UpdateFeeRatio {
            partner,
            admin: program_client.payer(),
        })
        .args(affiliate::instruction::UpdateFeeRatio { fee_ratio });

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}

pub fn fund_partner(
    program_client: &anchor_client::Program,
    vault: Pubkey,
    partner: String,
    amount: u64,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner)?;

    let funder_token = get_or_create_ata(program_client, token_mint, program_client.payer())?;
    let builder = program_client
        .request()
        .accounts(affiliate::accounts::FundPartner {
            partner,
            partner_token,
            funder_token,
            funder: program_client.payer(),
            token_program: spl_token::id(),
        })
        .args(affiliate::instruction::FundPartner { amount });

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}
