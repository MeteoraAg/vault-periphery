use crate::utils::{default_keypair, get_or_create_ata, simulate_transaction};
use anyhow::Result;
use hyper::Client;
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use solana_program::sysvar;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use std::fmt::Debug;
use std::ops::Deref;
use std::str::FromStr;
pub async fn init_partner<C: Deref<Target = impl Signer> + Clone>(
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

    let signature = builder.send().await?;
    println!("{}", signature);

    Ok(())
}

pub async fn update_fee_ratio<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
    partner: String,
    fee_ratio: u64,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault).await?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner).await?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner).await?;

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::UpdateFeeRatio {
            partner,
            admin: program_client.payer(),
        })
        .args(affiliate::instruction::UpdateFeeRatio { fee_ratio });

    let signature = builder.send().await?;
    println!("{}", signature);

    Ok(())
}

pub async fn fund_partner<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
    partner: String,
    amount: u64,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault).await?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner).await?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner).await?;

    let funder_token =
        get_or_create_ata(program_client, token_mint, program_client.payer()).await?;
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

    let signature = builder.send().await?;
    println!("{}", signature);

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct VaultList(Vec<VaultInfo>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VaultInfo {
    pub symbol: String,
    pub token_address: String,
}

pub async fn init_partner_all_vault<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    partner: String,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let url = "https://merv2-api.mercurial.finance/vault_info";

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let res = client.get(url.parse()?).await?;

    let buf = hyper::body::to_bytes(res).await?;

    let vault_list: VaultList = serde_json::from_slice(&buf)?;

    for vault in vault_list.0.iter() {
        let token_mint = Pubkey::from_str(&vault.token_address).unwrap();
        let (vault_pubkey, _) = Pubkey::find_program_address(
            &[
                b"vault".as_ref(),
                token_mint.as_ref(),
                mercurial_vault::get_base_key().as_ref(),
            ],
            &mercurial_vault::id(),
        );

        let partner_token = get_or_create_ata(program_client, token_mint, partner).await?;
        let (partner_pubkey, _nonce) = Pubkey::find_program_address(
            &[vault_pubkey.as_ref(), partner_token.as_ref()],
            &affiliate::id(),
        );
        // check whether partner is existed
        if program_client
            .rpc()
            .get_account_data(&partner_pubkey)
            .is_err()
        {
            println!(
                "init partner {} for vault {}",
                partner.to_string(),
                vault.symbol
            );
            init_partner(program_client, vault_pubkey, partner.to_string()).await?;
        } else {
            println!(
                "partner {} with vault {} is existed",
                partner.to_string(),
                vault.symbol
            );
        }
    }

    Ok(())
}
