//! Vault utilities

use anchor_lang::prelude::*;
use mercurial_vault::cpi::accounts::{DepositWithdrawLiquidity, WithdrawDirectlyFromStrategy};
use mercurial_vault::cpi::*;
use mercurial_vault::state::Vault;

/// Virtual price precision
pub const PRICE_PRECISION: u128 = 1_000_000_000_000u128;

/// MercurialVault struct
#[derive(Clone)]
pub struct MercurialVault;

impl anchor_lang::Id for MercurialVault {
    fn id() -> Pubkey {
        mercurial_vault::id()
    }
}
/// VaultUtils struct
pub struct VaultUtils;

impl VaultUtils {
    /// deposit to vault
    #[allow(clippy::too_many_arguments)]
    pub fn deposit<'info>(
        vault: &AccountInfo<'info>,
        lp_mint: &AccountInfo<'info>,
        user_token: &AccountInfo<'info>,
        user_lp: &AccountInfo<'info>,
        user: &AccountInfo<'info>,
        token_vault: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        vault_program: &AccountInfo<'info>,
        token_amount: u64,
        minimum_lp_amount: u64,
    ) -> Result<()> {
        let accounts = DepositWithdrawLiquidity {
            vault: vault.to_account_info(),
            lp_mint: lp_mint.to_account_info(),
            user_token: user_token.to_account_info(),
            user_lp: user_lp.to_account_info(),
            user: user.to_account_info(),
            token_vault: token_vault.to_account_info(),
            token_program: token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(vault_program.to_account_info(), accounts);
        deposit(cpi_ctx, token_amount, minimum_lp_amount)
    }
    /// withdraw from vault
    #[allow(clippy::too_many_arguments)]
    pub fn withdraw<'info>(
        vault: &AccountInfo<'info>,
        lp_mint: &AccountInfo<'info>,
        user_token: &AccountInfo<'info>,
        user_lp: &AccountInfo<'info>,
        user: &AccountInfo<'info>,
        token_vault: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        vault_program: &AccountInfo<'info>,
        unmint_amount: u64,
        minimum_out_amount: u64,
        signers: &[&[&[u8]]],
    ) -> Result<()> {
        let accounts = DepositWithdrawLiquidity {
            vault: vault.to_account_info(),
            lp_mint: lp_mint.to_account_info(),
            user_token: user_token.to_account_info(),
            user_lp: user_lp.to_account_info(),
            user: user.to_account_info(),
            token_vault: token_vault.to_account_info(),
            token_program: token_program.to_account_info(),
        };
        let cpi_ctx =
            CpiContext::new_with_signer(vault_program.to_account_info(), accounts, signers);

        withdraw(cpi_ctx, unmint_amount, minimum_out_amount)
    }
    /// withdraw directly from strategy
    #[allow(clippy::too_many_arguments)]
    pub fn withdraw_directly_from_strategy<'info>(
        vault: &AccountInfo<'info>,
        strategy: &AccountInfo<'info>,
        reserve: &AccountInfo<'info>,
        strategy_program: &AccountInfo<'info>,
        collateral_vault: &AccountInfo<'info>,
        token_vault: &AccountInfo<'info>,
        lp_mint: &AccountInfo<'info>,
        fee_vault: &AccountInfo<'info>,
        user_token: &AccountInfo<'info>,
        user_lp: &AccountInfo<'info>,
        user: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        vault_program: &AccountInfo<'info>,
        remaining_accounts: &[AccountInfo<'info>],
        unmint_amount: u64,
        minimum_out_amount: u64,
        signers: &[&[&[u8]]],
    ) -> Result<()> {
        let accounts = WithdrawDirectlyFromStrategy {
            vault: vault.clone(),
            strategy: strategy.clone(),
            reserve: reserve.clone(),
            strategy_program: strategy_program.clone(),
            collateral_vault: collateral_vault.clone(),
            token_vault: token_vault.clone(),
            lp_mint: lp_mint.clone(),
            fee_vault: fee_vault.clone(),
            user_token: user_token.clone(),
            user_lp: user_lp.clone(),
            user: user.clone(),
            token_program: token_program.clone(),
        };

        let cpi_ctx =
            CpiContext::new_with_signer(vault_program.to_account_info(), accounts, signers)
                .with_remaining_accounts(remaining_accounts.to_vec());

        withdraw_directly_from_strategy(cpi_ctx, unmint_amount, minimum_out_amount)
    }
}
/// VirtualPrice trait
pub trait VirtualPrice {
    /// get virtual price
    fn get_virtual_price(&self, current_time: u64, lp_supply: u64) -> Option<u64>;
}

impl VirtualPrice for Vault {
    fn get_virtual_price(&self, current_time: u64, lp_supply: u64) -> Option<u64> {
        // When the vault is newly created, or the vault liquidity is fully withdrawn
        if lp_supply == 0 {
            return u64::try_from(PRICE_PRECISION).ok(); // virtual price = 1
        }
        let unlocked_amount = self.get_unlocked_amount(current_time)?;
        let virtual_price = u128::from(unlocked_amount)
            .checked_mul(PRICE_PRECISION)?
            .checked_div(u128::from(lp_supply))?;
        u64::try_from(virtual_price).ok()
    }
}
