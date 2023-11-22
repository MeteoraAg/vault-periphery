use crate::utils;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use bincode::deserialize;
use solana_program::sysvar;
use solana_program::sysvar::clock::Clock;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::convert::TryFrom;
use std::ops::Deref;

pub async fn show<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
) -> Result<()> {
    let vault_data: mercurial_vault::state::Vault = program_client.account(vault).await?;
    println!("VAULT DATA: {:#?}", vault_data);
    let token_mint: anchor_spl::token::Mint = program_client.account(vault_data.lp_mint).await?;

    let current_timestamp = get_current_node_clock_time(program_client)?;

    println!(
        "TOTAL_AMOUNT: {}, TOTAL_UNLOCKED_AMOUNT: {}, lp_mint {}",
        vault_data.total_amount,
        vault_data.get_unlocked_amount(current_timestamp).unwrap(),
        token_mint.supply
    );

    let token_data: anchor_spl::token::TokenAccount =
        program_client.account(vault_data.token_vault).await?;

    println!("TOKEN AMOUNT: {}", token_data.amount);

    let mut strategy_amount = 0u64;
    for (i, &strategy_pubkey) in vault_data.strategies.iter().enumerate() {
        if strategy_pubkey != Pubkey::default() {
            let strategy_state: mercurial_vault::state::Strategy =
                program_client.account(strategy_pubkey).await?;

            println!("STRATEGY DATA {}: {:#?}", strategy_pubkey, strategy_state);

            strategy_amount += strategy_state.current_liquidity;
        }
    }
    assert_eq!(vault_data.total_amount, token_data.amount + strategy_amount);
    println!("Ok");
    Ok(())
}

pub fn get_current_node_clock_time<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
) -> Result<u64> {
    let rpc = program_client.rpc();
    let clock_account = rpc.get_account(&sysvar::clock::id())?;
    let clock = deserialize::<Clock>(&clock_account.data)?;
    let current_time = u64::try_from(clock.unix_timestamp)?;
    Ok(current_time)
}

pub fn get_unlocked_amount<C: Deref<Target = impl Signer> + Clone>(
    program_client: &anchor_client::Program<C>,
    vault: Pubkey,
    payer: &Keypair,
) -> Result<()> {
    let builder = program_client
        .request()
        .accounts(mercurial_vault::accounts::GetUnlockedAmount { vault })
        .args(mercurial_vault::instruction::GetUnlockedAmount {});

    let simulation = utils::simulate_transaction(&builder, &program_client, &vec![payer]).unwrap();
    let logs = simulation.value.logs.expect("No log in simulation found");
    let unlocked_amount: mercurial_vault::TotalAmount =
        utils::parse_event_log(&logs).expect("Event log not found");
    println!("UNLOCKED AMOUNT: {}", unlocked_amount.total_amount);
    Ok(())
}
