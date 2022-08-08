use crate::helpers::utils;
use crate::helpers::utils::AddPacked;
use affiliate::vault_utils;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_lang::solana_program::{
    program_option::COption,
    program_pack::{Pack, Sealed},
    pubkey::Pubkey,
};
use anchor_lang::{
    AccountDeserialize, AccountSerialize, AnchorSerialize, Discriminator, InstructionData,
    ToAccountMetas,
};
use mercurial_vault::context::VaultBumps;
use mercurial_vault::state::Vault;
use solana_program_test::*;
use solana_sdk::account::AccountSharedData;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::native_token::LAMPORTS_PER_SOL;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::{Account as Token, AccountState, Mint};

use super::utils::{get_anchor_seder_account, get_packable_seder_account, process_and_assert_ok};

#[derive(Debug, Copy, Clone)]
pub struct Strategy {
    pub strategy_pubkey: Pubkey,
    pub reserve_pubkey: Pubkey,
    pub collateral_vault_pubkey: Pubkey,
    pub strategy_program_pubkey: Pubkey,
    pub reserve_liquidity_supply: Pubkey,
    pub lending_market: Pubkey,
    pub lending_market_authority: Pubkey,
    pub reserve_collateral_mint: Pubkey,
}

#[derive(Debug, Copy, Clone)]
pub struct AddVaultRes {
    pub vault_pubkey: Pubkey,
    pub token_mint_pubkey: Pubkey,
    pub lp_mint_pubkey: Pubkey,
    pub operator_pubkey: Pubkey,
    pub fee_vault_pubkey: Pubkey,
    pub admin_pubkey: Pubkey,
    pub token_vault_pubkey: Pubkey,
}

#[derive(Clone, Debug)]
pub struct PackableVault(Vault);

impl Sealed for PackableVault {}

impl Pack for PackableVault {
    // Discriminator + Vault
    const LEN: usize = 8 + std::mem::size_of::<Vault>();
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let discriminator = Vault::discriminator();
        let buf = self.0.try_to_vec().unwrap();
        for i in 0..discriminator.len() {
            dst[i] = discriminator[i];
        }
        for i in 0..buf.len() {
            dst[i + discriminator.len()] = buf[i];
        }
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut src = src.clone();
        Ok(PackableVault(Vault::try_deserialize(&mut src).unwrap()))
    }
    fn get_packed_len() -> usize {
        Self::LEN
    }
}

pub fn add_vault(
    test: &mut ProgramTest,
    token_mint_pubkey: Pubkey,
    token_decimal: u8,
) -> AddVaultRes {
    let admin_pubkey = Pubkey::new_unique();
    let base_pubkey = mercurial_vault::get_base_key();
    let fee_vault_pubkey = Pubkey::new_unique();
    let lp_mint_pubkey = Pubkey::new_unique();
    let operator_pubkey = Pubkey::new_unique();

    let seeds = &[
        b"vault".as_ref(),
        token_mint_pubkey.as_ref(),
        base_pubkey.as_ref(),
    ];
    let (vault_pubkey, vault_bump) = Pubkey::find_program_address(seeds, &mercurial_vault::id());

    let seeds = &[b"token_vault".as_ref(), vault_pubkey.as_ref()];
    let (token_vault_pubkey, token_vault_bump) =
        Pubkey::find_program_address(seeds, &mercurial_vault::id());

    test.add_packable_account(
        lp_mint_pubkey,
        10 * LAMPORTS_PER_SOL,
        &Mint {
            is_initialized: true,
            mint_authority: COption::Some(vault_pubkey),
            decimals: token_decimal,
            ..Mint::default()
        },
        &spl_token::id(),
    );

    test.add_packable_account(
        token_vault_pubkey,
        10 * LAMPORTS_PER_SOL,
        &Token {
            mint: token_mint_pubkey,
            owner: vault_pubkey,
            state: AccountState::Initialized,
            ..Token::default()
        },
        &spl_token::id(),
    );

    test.add_packable_account(
        fee_vault_pubkey,
        10 * LAMPORTS_PER_SOL,
        &Token {
            mint: token_mint_pubkey,
            owner: vault_pubkey,
            state: AccountState::Initialized,
            ..Token::default()
        },
        &spl_token::id(),
    );

    let vault = Vault {
        enabled: 1_u8,
        bumps: VaultBumps {
            vault_bump,
            token_vault_bump,
        },
        token_vault: token_vault_pubkey,
        fee_vault: fee_vault_pubkey,
        token_mint: token_mint_pubkey,
        lp_mint: lp_mint_pubkey,
        base: base_pubkey,
        admin: admin_pubkey,
        operator: operator_pubkey,
        ..Vault::default()
    };

    let test_vault = PackableVault(vault);
    test.add_packable_account(
        vault_pubkey,
        10 * LAMPORTS_PER_SOL,
        &test_vault,
        &mercurial_vault::id(),
    );

    AddVaultRes {
        fee_vault_pubkey,
        lp_mint_pubkey,
        operator_pubkey,
        token_mint_pubkey,
        vault_pubkey,
        admin_pubkey,
        token_vault_pubkey,
    }
}

pub async fn deposit_to_strategy(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    strategy: Strategy,
    vault: AddVaultRes,
) {
    let vault_state = utils::get_anchor_seder_account::<mercurial_vault::state::Vault>(
        &vault.vault_pubkey,
        banks_client,
    )
    .await;
    let mut accounts = mercurial_vault::accounts::RebalanceStrategy {
        collateral_vault: strategy.collateral_vault_pubkey,
        fee_vault: vault.fee_vault_pubkey,
        lp_mint: vault.lp_mint_pubkey,
        operator: payer.pubkey(),
        reserve: strategy.reserve_pubkey,
        strategy: strategy.strategy_pubkey,
        vault: vault.vault_pubkey,
        strategy_program: strategy.strategy_program_pubkey,
        token_program: spl_token::id(),
        token_vault: vault.token_vault_pubkey,
    }
    .to_account_metas(None);
    let mut remaining_accounts = vec![
        AccountMeta::new(strategy.reserve_liquidity_supply, false),
        AccountMeta::new_readonly(strategy.lending_market, false),
        AccountMeta::new_readonly(strategy.lending_market_authority, false),
        AccountMeta::new(strategy.reserve_collateral_mint, false),
        AccountMeta::new_readonly(anchor_lang::solana_program::sysvar::clock::id(), false),
    ];
    accounts.append(&mut remaining_accounts);
    let ix = Instruction {
        accounts,
        data: mercurial_vault::instruction::DepositStrategy {
            amount: vault_state.total_amount,
        }
        .data(),
        program_id: mercurial_vault::id(),
    };
    process_and_assert_ok(&[ix], payer, &[payer], banks_client).await;
}

pub async fn get_onchain_time(banks_client: &mut BanksClient) -> u64 {
    let clock_account = banks_client
        .get_account(solana_program::sysvar::clock::id())
        .await
        .unwrap()
        .unwrap();

    let clock_state =
        bincode::deserialize::<solana_program::clock::Clock>(clock_account.data.as_ref()).unwrap();

    clock_state.unix_timestamp as u64
}

pub async fn get_virtual_price(banks_client: &mut BanksClient, vault: AddVaultRes) -> u64 {
    let lp_mint_state =
        get_packable_seder_account::<spl_token::state::Mint>(&vault.lp_mint_pubkey, banks_client)
            .await;

    let vault_state = get_anchor_seder_account::<mercurial_vault::state::Vault>(
        &vault.vault_pubkey,
        banks_client,
    )
    .await;

    let onchain_time = get_onchain_time(banks_client).await;
    let total_amount = vault_state.get_unlocked_amount(onchain_time).unwrap();

    let virtual_price =
        total_amount as u128 * vault_utils::PRICE_PRECISION / lp_mint_state.supply as u128;

    virtual_price as u64
}

pub async fn add_strategy(
    context: &mut ProgramTestContext,
    strategy: &mut Strategy,
    vault: AddVaultRes,
) {
    let (strategy_pubkey, _strategy_bump) = Pubkey::find_program_address(
        &[
            vault.vault_pubkey.as_ref(),
            strategy.reserve_pubkey.as_ref(),
            &[0],
        ],
        &mercurial_vault::id(),
    );

    let (collateral_vault, _collateral_vault_bump) = Pubkey::find_program_address(
        &["collateral_vault".as_ref(), strategy_pubkey.as_ref()],
        &mercurial_vault::id(),
    );

    strategy.strategy_pubkey = strategy_pubkey;
    strategy.collateral_vault_pubkey = collateral_vault;

    let collateral_vault_state = spl_token::state::Account {
        mint: strategy.reserve_collateral_mint,
        amount: 0u64,
        owner: vault.vault_pubkey,
        delegated_amount: 0u64,
        is_native: COption::None,
        close_authority: COption::None,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
    };

    let mut collateral_vault_bytes = [0u8; spl_token::state::Account::LEN];
    spl_token::state::Account::pack(collateral_vault_state, &mut collateral_vault_bytes).unwrap();
    let mut collateral_vault_account = AccountSharedData::new(
        10 * LAMPORTS_PER_SOL,
        collateral_vault_bytes.len(),
        &spl_token::id(),
    );
    collateral_vault_account.set_data(collateral_vault_bytes.to_vec());
    context.set_account(&collateral_vault, &collateral_vault_account);

    let strategy_state = mercurial_vault::state::Strategy {
        bumps: [0u8; 10],
        collateral_vault,
        current_liquidity: 0,
        reserve: strategy.reserve_pubkey,
        strategy_type: mercurial_vault::strategy::base::StrategyType::SolendWithoutLM,
        vault: vault.vault_pubkey,
    };

    let mut strategy_state_bytes = vec![];
    mercurial_vault::state::Strategy::try_serialize(&strategy_state, &mut strategy_state_bytes)
        .unwrap();
    let mut strategy_account = AccountSharedData::new(
        10 * LAMPORTS_PER_SOL,
        strategy_state_bytes.len(),
        &mercurial_vault::id(),
    );
    strategy_account.set_data(strategy_state_bytes);
    context.set_account(&strategy_pubkey, &strategy_account);

    let mut vault_account = context
        .banks_client
        .get_account(vault.vault_pubkey)
        .await
        .unwrap()
        .unwrap();

    let mut vault_state =
        mercurial_vault::state::Vault::try_deserialize(&mut vault_account.data.as_ref()).unwrap();

    vault_state.strategies[0] = strategy_pubkey;

    let mut vault_state_bytes = vec![];
    mercurial_vault::state::Vault::try_serialize(&mut vault_state, &mut vault_state_bytes).unwrap();

    vault_account.data = vault_state_bytes;
    context.set_account(&vault.vault_pubkey, &AccountSharedData::from(vault_account));
}
