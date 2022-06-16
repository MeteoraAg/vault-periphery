use crate::strategy_handler::base::StrategyHandler;
use crate::strategy_handler::mango_adapter::{
    MangoGroupAdapter, MangoNodeBankAdapter, MangoRootBankAdapter,
};
use crate::user::get_or_create_ata;
use anchor_lang::InstructionData;
use anchor_lang::ToAccountMetas;
use anyhow::Result;
use mercurial_vault::strategy::base::{get_mango_group_id, get_mango_program_id};

use solana_program::pubkey::Pubkey;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::instruction::Instruction;
use std::str::FromStr;
pub struct MangoHandler {}

impl StrategyHandler for MangoHandler {
    fn withdraw_directly_from_strategy(
        &self,
        program_client: &anchor_client::Program,
        strategy: Pubkey,
        token_mint: Pubkey,
        base: Pubkey,
        partner: String,
        amount: u64,
    ) -> Result<()> {
        let (vault, _vault_bump) = Pubkey::find_program_address(
            &[
                mercurial_vault::seed::VAULT_PREFIX.as_ref(),
                token_mint.as_ref(),
                base.as_ref(),
            ],
            &mercurial_vault::id(),
        );

        let (token_vault, _token_vault_bump) = Pubkey::find_program_address(
            &[
                mercurial_vault::seed::TOKEN_VAULT_PREFIX.as_ref(),
                vault.as_ref(),
            ],
            &mercurial_vault::id(),
        );
        let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
        let strategy_state: mercurial_vault::state::Strategy = program_client.account(strategy)?;

        let root_bank_pk = strategy_state.reserve;

        let program_id = get_mango_program_id();
        let mango_group_pk = get_mango_group_id();
        let owner_pk = vault;
        let (mango_account_pk, _bump) = Pubkey::find_program_address(
            &[
                &mango_group_pk.as_ref(),
                &owner_pk.as_ref(),
                &0u64.to_le_bytes(),
            ],
            &program_id,
        );

        let mango_group_state: MangoGroupAdapter = program_client.account(mango_group_pk)?;
        let mango_cache_pk = mango_group_state.mango_cache;

        let root_bank_state: MangoRootBankAdapter = program_client.account(root_bank_pk)?;
        let node_bank_pk = root_bank_state.node_banks[0];
        let node_bank_state: MangoNodeBankAdapter = program_client.account(node_bank_pk)?;
        let vault_pk = node_bank_state.vault;

        let signer_pk = mango_group_state.signer_key;

        let (collateral_vault, _collateral_vault_bump) = Pubkey::find_program_address(
            &[
                mercurial_vault::seed::COLLATERAL_VAULT_PREFIX.as_ref(),
                strategy.as_ref(),
            ],
            &mercurial_vault::id(),
        );

        let lp_mint = vault_state.lp_mint;
        let user_token = get_or_create_ata(program_client, token_mint, program_client.payer())?;

        let partner = Pubkey::from_str(&partner).unwrap();
        let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
        let (partner, _nonce) = Pubkey::find_program_address(
            &[vault.as_ref(), partner_token.as_ref()],
            &affiliate::id(),
        );
        // check whether partner is existed
        let _partner_state: affiliate::Partner = program_client.account(partner)?;
        let (user, _nonce) = Pubkey::find_program_address(
            &[partner.as_ref(), program_client.payer().as_ref()],
            &affiliate::id(),
        );
        // check whether user is existed
        let _user_state: affiliate::User = program_client.account(user)?;

        let user_lp = get_or_create_ata(program_client, lp_mint, user)?;

        let mut accounts = affiliate::accounts::WithdrawDirectlyFromStrategy {
            partner,
            user,
            vault,
            vault_program: mercurial_vault::id(),
            strategy,
            reserve: strategy_state.reserve,
            strategy_program: get_mango_program_id(),
            collateral_vault,
            token_vault,
            fee_vault: vault_state.fee_vault,
            vault_lp_mint: lp_mint,
            user_token,
            user_lp,
            owner: program_client.payer(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None);
        let mut remaining_accounts = vec![
            AccountMeta::new(mango_group_pk, false),
            AccountMeta::new(mango_account_pk, false),
            AccountMeta::new(mango_cache_pk, false),
            AccountMeta::new(node_bank_pk, false),
            AccountMeta::new(vault_pk, false),
            AccountMeta::new(signer_pk, false),
            AccountMeta::new_readonly(Pubkey::default(), false),
        ];

        accounts.append(&mut remaining_accounts);

        let instructions = vec![Instruction {
            program_id: affiliate::id(),
            accounts,
            data: affiliate::instruction::WithdrawDirectlyFromStrategy {
                unmint_amount: amount,
                min_out_amount: 0,
            }
            .data(),
        }];

        let builder = program_client.request();
        let builder = instructions
            .into_iter()
            .fold(builder, |bld, ix| bld.instruction(ix));

        let signature = builder.send()?;
        println!("{}", signature);
        Ok(())
    }
}
