mod admin;
mod partner;
mod strategy_handler;
mod user;
mod utils;
mod vault;

use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Client;
use anchor_client::Cluster;
use anyhow::Result;

use clap::Parser;
use mercurial_vault::get_base_key;
use solana_sdk::signature::{read_keypair_file, Keypair};
use strategy_handler::base::get_strategy_handler;

use crate::utils::default_keypair;
use admin::*;
use partner::*;
use std::rc::Rc;
use std::str::FromStr;
use user::*;
use vault::*;

#[derive(Default, Debug, Parser)]
pub struct ConfigOverride {
    /// Cluster override.
    #[clap(global = true, long = "provider.cluster")]
    pub cluster: Option<Cluster>,
    /// Wallet override.
    #[clap(global = true, long = "provider.wallet")]
    pub wallet: Option<String>,

    /// Program id override
    #[clap(global = true, long = "provider.program_id")]
    pub program_id: Option<String>,

    /// Token mint override
    #[clap(global = true, long = "provider.token_mint")]
    pub token_mint: Option<String>,

    #[clap(global = true, long = "provider.base")]
    pub base: Option<String>,
}

#[derive(Debug, Parser)]
pub enum Command {
    #[clap(flatten)]
    Vault(VaultCommand),
    #[clap(flatten)]
    User(UserCommand),
    #[clap(flatten)]
    Partner(PartnerCommand),
    #[clap(flatten)]
    Admin(AdminCommand),
}

#[derive(Debug, Parser)]
pub enum VaultCommand {
    Show {},
    GetUnlockedAmount {},
}

#[derive(Debug, Parser)]
pub enum UserCommand {
    Deposit {
        token_amount: u64,
        partner: String,
    },
    Withdraw {
        unmint_amount: u64,
        partner: String,
    },
    WithdrawFromStrategy {
        unmint_amount: u64,
        strategy: Pubkey,
        partner: String,
    },
    ViewUser {
        partner: String,
    },
}

#[derive(Debug, Parser)]
pub enum AdminCommand {
    InitPartner { partner: String },
    UpdateFeeRatio { partner: String, fee_ratio: u64 },
    FundPartner { partner: String, amount: u64 },
}

#[derive(Debug, Parser)]
pub enum PartnerCommand {
    InitUser { partner: String },
    ViewPartner { partner: String },
}

#[derive(Parser)]
pub struct Opts {
    #[clap(flatten)]
    pub cfg_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}
fn main() -> Result<()> {
    let opts = Opts::parse();

    let payer = match opts.cfg_override.wallet {
        Some(wallet) => read_keypair_file(wallet).expect("Requires a keypair file"),
        None => default_keypair(),
    };
    let url = match opts.cfg_override.cluster {
        Some(cluster) => cluster,
        None => Cluster::Devnet,
    };

    let client = Client::new_with_options(
        url,
        Rc::new(Keypair::from_bytes(&payer.to_bytes())?),
        CommitmentConfig::processed(),
    );

    let program_id = match opts.cfg_override.program_id {
        Some(program_id) => Pubkey::from_str(&program_id).unwrap(),
        None => affiliate::id(),
    };

    let program_client = client.program(program_id);

    let token_mint = match opts.cfg_override.token_mint {
        Some(token_mint) => Pubkey::from_str(&token_mint).unwrap(),
        None => Pubkey::default(),
    };

    let base = match opts.cfg_override.base {
        Some(base) => Pubkey::from_str(&base).unwrap(),
        None => get_base_key(),
    };

    let (vault, _) = Pubkey::find_program_address(
        &[b"vault".as_ref(), token_mint.as_ref(), base.as_ref()],
        &mercurial_vault::id(),
    );

    println!("ProgramID {}", program_id.to_string());
    println!("TOKEN MINT {}", token_mint);
    println!("Base {}", base);
    println!("VAULT {}", vault);

    // Fee payer is the admin
    match opts.command {
        Command::Vault(vault_command) => match vault_command {
            VaultCommand::Show {} => show(&program_client, vault)?,
            VaultCommand::GetUnlockedAmount {} => {
                get_unlocked_amount(&program_client, vault, &payer)?
            }
        },
        Command::User(user) => match user {
            UserCommand::Deposit {
                token_amount,
                partner,
            } => deposit(&program_client, token_mint, base, partner, token_amount)?,
            UserCommand::Withdraw {
                unmint_amount,
                partner,
            } => withdraw(&program_client, token_mint, base, partner, unmint_amount)?,
            UserCommand::WithdrawFromStrategy {
                unmint_amount,
                strategy,
                partner,
            } => {
                let strategy_state: mercurial_vault::state::Strategy =
                    program_client.account(strategy)?;

                let strategy_handler = get_strategy_handler(strategy_state.strategy_type);
                strategy_handler.withdraw_directly_from_strategy(
                    &program_client,
                    strategy,
                    token_mint,
                    base,
                    partner,
                    unmint_amount,
                )?
            }
            UserCommand::ViewUser { partner } => view_user(&program_client, vault, partner)?,
        },
        Command::Partner(partner) => match partner {
            PartnerCommand::InitUser { partner } => init_user(&program_client, vault, partner)?,
            PartnerCommand::ViewPartner { partner } => {
                view_partner(&program_client, vault, partner)?
            }
        },
        Command::Admin(admin) => match admin {
            AdminCommand::InitPartner { partner } => init_partner(&program_client, vault, partner)?,
            AdminCommand::UpdateFeeRatio { partner, fee_ratio } => {
                update_fee_ratio(&program_client, vault, partner, fee_ratio)?
            }
            AdminCommand::FundPartner { partner, amount } => {
                fund_partner(&program_client, vault, partner, amount)?
            }
        },
    };

    Ok(())
}
