use crate::utils::get_or_create_ata;
use anyhow::Result;
use solana_program::sysvar;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use std::ops::Deref;
use std::str::FromStr;
// must be called by user
pub async fn init_user<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
    partner: String,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();

    let vault_state: mercurial_vault::state::Vault = program_client.account(vault).await?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner).await?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    println!("{} {}", partner, partner_token);
    // return Ok(());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner).await?;

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

    let signature = builder.send().await?;
    println!("{}", signature);

    Ok(())
}

pub async fn init_user_permissionless<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
    partner: String,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();

    let vault_state: mercurial_vault::state::Vault = program_client.account(vault).await?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner).await?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    println!("{} {}", partner, partner_token);
    // return Ok(());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner).await?;

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

    let signature = builder.send().await?;
    println!("{}", signature);

    Ok(())
}

pub async fn view_partner<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
    partner: String,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault).await?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner).await?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let partner_state: affiliate::Partner = program_client.account(partner).await?;
    println!("{:?}", partner_state);
    Ok(())
}
