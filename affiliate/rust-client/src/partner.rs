use crate::utils::get_or_create_ata;
use anyhow::Result;
use solana_program::sysvar;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use std::str::FromStr;
// must be called by user
pub fn init_user(
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
    println!("{} {}", partner, partner_token);
    // return Ok(());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner)?;

    let (user, _nonce) = Pubkey::find_program_address(
        &[partner.as_ref(), program_client.payer().as_ref()],
        &affiliate::id(),
    );

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::InitUser {
            user,
            partner,
            owner: program_client.payer(),
            system_program: system_program::id(),
            rent: sysvar::rent::ID,
        })
        .args(affiliate::instruction::InitUser {});

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}

pub fn init_user_permissionless(
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
    println!("{} {}", partner, partner_token);
    // return Ok(());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner)?;

    let (user, _nonce) = Pubkey::find_program_address(
        &[partner.as_ref(), program_client.payer().as_ref()],
        &affiliate::id(),
    );

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::InitUserPermissionless {
            user,
            partner,
            owner: program_client.payer(),
            payer: program_client.payer(),
            system_program: system_program::id(),
        })
        .args(affiliate::instruction::InitUserPermissionless {});

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}

pub fn view_partner(
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
    // check whether partner is existed
    let partner_state: affiliate::Partner = program_client.account(partner)?;
    println!("{:?}", partner_state);
    Ok(())
}
