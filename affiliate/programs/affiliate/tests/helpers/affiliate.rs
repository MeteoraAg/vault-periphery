use super::utils::*;
use anchor_lang::prelude::Pubkey;
use anchor_lang::{
    system_program, AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas,
};
use solana_program_test::{BanksClient, ProgramTestContext};
use solana_sdk::account::AccountSharedData;
use solana_sdk::program_pack::Pack;
use solana_sdk::sysvar;
use solana_sdk::{instruction::Instruction, signature::Keypair, signer::Signer};

use super::vault::AddVaultRes;

#[derive(Debug)]
pub struct SimulateInterestRes {
    pub performance_fee: u64,
    pub partner_fee: u64,
}

pub async fn update_fee_ratio(
    banks_client: &mut BanksClient,
    partner_pda: Pubkey,
    fee_ratio: u64,
    payer: &Keypair,
) {
    let admin_keypair = get_admin_keypair();
    let accounts = affiliate::accounts::UpdateFeeRatio {
        admin: admin_keypair.pubkey(),
        partner: partner_pda,
    }
    .to_account_metas(None);
    let ix = Instruction {
        accounts,
        data: affiliate::instruction::UpdateFeeRatio { fee_ratio }.data(),
        program_id: affiliate::id(),
    };
    process_and_assert_ok(&[ix], payer, &[&admin_keypair, payer], banks_client).await;
}

pub async fn withdraw(
    banks_client: &mut BanksClient,
    vault: AddVaultRes,
    user_wallet: &Keypair,
    amount: u64,
    partner_pda: Pubkey,
    user_pda: Pubkey,
    payer: &Keypair,
) -> Pubkey {
    let user_token_ata = get_or_create_ata(
        payer,
        &vault.token_mint_pubkey,
        &user_wallet.pubkey(),
        banks_client,
    )
    .await;

    let user_lp = get_or_create_ata(payer, &vault.lp_mint_pubkey, &user_pda, banks_client).await;

    let accounts = affiliate::accounts::DepositWithdrawLiquidity {
        owner: user_wallet.pubkey(),
        partner: partner_pda,
        token_program: spl_token::id(),
        token_vault: vault.token_vault_pubkey,
        user: user_pda,
        user_lp,
        user_token: user_token_ata,
        vault: vault.vault_pubkey,
        vault_lp_mint: vault.lp_mint_pubkey,
        vault_program: mercurial_vault::id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        accounts,
        data: affiliate::instruction::Withdraw {
            min_out_amount: 0,
            unmint_amount: amount,
        }
        .data(),
        program_id: affiliate::id(),
    };

    process_and_assert_ok(&[ix], payer, &[&payer, &user_wallet], banks_client).await;

    user_lp
}

pub async fn deposit(
    banks_client: &mut BanksClient,
    vault: AddVaultRes,
    user_wallet: &Keypair,
    amount: u64,
    partner_pda: Pubkey,
    user_pda: Pubkey,
    payer: &Keypair,
) -> Pubkey {
    let user_token_ata = get_or_create_ata(
        payer,
        &vault.token_mint_pubkey,
        &user_wallet.pubkey(),
        banks_client,
    )
    .await;

    let user_lp = get_or_create_ata(payer, &vault.lp_mint_pubkey, &user_pda, banks_client).await;

    let accounts = affiliate::accounts::DepositWithdrawLiquidity {
        owner: user_wallet.pubkey(),
        partner: partner_pda,
        token_program: spl_token::id(),
        token_vault: vault.token_vault_pubkey,
        user: user_pda,
        user_lp,
        user_token: user_token_ata,
        vault: vault.vault_pubkey,
        vault_lp_mint: vault.lp_mint_pubkey,
        vault_program: mercurial_vault::id(),
    }
    .to_account_metas(None);

    let ix = Instruction {
        accounts,
        data: affiliate::instruction::Deposit {
            minimum_lp_token_amount: 0,
            token_amount: amount,
        }
        .data(),
        program_id: affiliate::id(),
    };

    process_and_assert_ok(&[ix], payer, &[&payer, &user_wallet], banks_client).await;

    user_lp
}

pub async fn simulate_interest(
    context: &mut ProgramTestContext,
    vault: AddVaultRes,
    interest_amount: u64,
) {
    let mut vault_account = context
        .banks_client
        .get_account(vault.vault_pubkey)
        .await
        .unwrap()
        .unwrap();

    let mut vault_state =
        mercurial_vault::state::Vault::try_deserialize(&mut vault_account.data.as_ref()).unwrap();

    vault_state.total_amount += interest_amount;

    let mut new_vault_state_bytes = vec![];
    mercurial_vault::state::Vault::try_serialize(&vault_state, &mut new_vault_state_bytes).unwrap();
    vault_account.data = new_vault_state_bytes;

    context.set_account(&vault.vault_pubkey, &AccountSharedData::from(vault_account));

    let mut token_vault = context
        .banks_client
        .get_account(vault.token_vault_pubkey)
        .await
        .unwrap()
        .unwrap();

    let mut token_vault_state =
        spl_token::state::Account::unpack(token_vault.data.as_ref()).unwrap();

    token_vault_state.amount += interest_amount;

    let new_token_vault_state_bytes = &mut [0u8; spl_token::state::Account::LEN];
    spl_token::state::Account::pack(token_vault_state, new_token_vault_state_bytes).unwrap();

    token_vault.data = new_token_vault_state_bytes.to_vec();

    context.set_account(
        &vault.token_vault_pubkey,
        &AccountSharedData::from(token_vault),
    );
}

pub async fn init_user(
    banks_client: &mut BanksClient,
    user_wallet: &Keypair,
    partner_pda: Pubkey,
    payer: &Keypair,
) -> Pubkey {
    let (user_pda, _user_bump) = Pubkey::find_program_address(
        &[&partner_pda.as_ref(), &user_wallet.pubkey().as_ref()],
        &affiliate::id(),
    );

    let accounts = affiliate::accounts::InitUser {
        owner: user_wallet.pubkey(),
        partner: partner_pda,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
        user: user_pda,
    }
    .to_account_metas(None);
    let data = affiliate::instruction::InitUser {}.data();
    let ix = Instruction {
        accounts,
        data,
        program_id: affiliate::id(),
    };
    process_and_assert_ok(&[ix], payer, &[&user_wallet, &payer], banks_client).await;

    user_pda
}

pub async fn init_partner(
    banks_client: &mut BanksClient,
    vault: AddVaultRes,
    partner_wallet: Pubkey,
    payer: &Keypair,
) -> Pubkey {
    let admin_keypair = get_admin_keypair();
    let partner_token = get_or_create_ata(
        payer,
        &vault.token_mint_pubkey,
        &partner_wallet,
        banks_client,
    )
    .await;
    let (partner_pda, _partner_bump) = Pubkey::find_program_address(
        &[&vault.vault_pubkey.as_ref(), &partner_token.as_ref()],
        &affiliate::id(),
    );
    let accounts = affiliate::accounts::InitPartner {
        admin: admin_keypair.pubkey(),
        partner: partner_pda,
        partner_token,
        rent: sysvar::rent::id(),
        system_program: system_program::ID,
        token_program: spl_token::id(),
        vault: vault.vault_pubkey,
    }
    .to_account_metas(None);
    let init_partner_ix = Instruction {
        accounts,
        data: affiliate::instruction::InitPartner {}.data(),
        program_id: affiliate::id(),
    };
    process_and_assert_ok(
        &[init_partner_ix],
        payer,
        &[&admin_keypair, &payer],
        banks_client,
    )
    .await;

    partner_pda
}
