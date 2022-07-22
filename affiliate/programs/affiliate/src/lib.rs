//! Affiliate program
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::integer_arithmetic)]
#![warn(missing_docs)]

use anchor_lang::prelude::*;
pub mod vault_utils;
use crate::vault_utils::PRICE_PRECISION;
use crate::vault_utils::{MercurialVault, VaultUtils, VirtualPrice};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use mercurial_vault::state::Vault;
use mercurial_vault::{PERFORMANCE_FEE_DENOMINATOR, PERFORMANCE_FEE_NUMERATOR};
use std::str::FromStr;
use vipers::prelude::*;

declare_id!("GacY9YuN16HNRTy7ZWwULPccwvfFSBeNLuAQP7y38Du3");

/// Admin address, only admin can initialize a partner
pub fn get_admin_address() -> Pubkey {
    Pubkey::from_str("DHLXnJdACTY83yKwnUkeoDjqi4QBbsYGa1v8tJL76ViX")
        .expect("Must be correct Solana address")
}

/// Fee denominator
const FEE_DENOMINATOR: u128 = 10_000;
const DEFAULT_FEE_RATIO: u64 = 5_000; // 50%

/// affiliate program
#[program]
pub mod affiliate {
    use super::*;
    /// function can be only called by admin
    pub fn init_partner(ctx: Context<InitPartner>) -> Result<()> {
        let partner = &mut ctx.accounts.partner;
        partner.vault = ctx.accounts.vault.key();
        partner.partner_token = ctx.accounts.partner_token.key();
        partner.fee_ratio = DEFAULT_FEE_RATIO;
        Ok(())
    }

    /// function can be only called by admin
    pub fn update_fee_ratio(ctx: Context<UpdateFeeRatio>, fee_ratio: u64) -> Result<()> {
        let partner = &mut ctx.accounts.partner;
        if fee_ratio > FEE_DENOMINATOR as u64 {
            return Err(VaultError::InvalidFeeRatio.into());
        }
        partner.fee_ratio = fee_ratio;
        Ok(())
    }

    /// function can be only called by user
    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.partner = ctx.accounts.partner.key();
        user.owner = ctx.accounts.owner.key();
        user.bump = unwrap_bump!(ctx, "user");
        Ok(())
    }

    /// deposit
    #[allow(clippy::needless_lifetimes)]
    pub fn deposit<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DepositWithdrawLiquidity>,
        token_amount: u64,
        minimum_lp_token_amount: u64,
    ) -> Result<()> {
        let vault = &ctx.accounts.vault.to_account_info();
        let vault_lp_mint = &ctx.accounts.vault_lp_mint.to_account_info();
        let user_lp = &ctx.accounts.user_lp.to_account_info();

        let user_token = &ctx.accounts.user_token.to_account_info();
        let token_vault = &ctx.accounts.token_vault.to_account_info();
        let token_program = &ctx.accounts.token_program.to_account_info();
        let vault_program = &ctx.accounts.vault_program.to_account_info();
        let owner = &ctx.accounts.owner.to_account_info();
        update_liquidity_wrapper(
            move || {
                VaultUtils::deposit(
                    vault,
                    vault_lp_mint,
                    user_token,
                    user_lp, // mint vault lp token to pool lp token account
                    owner,
                    token_vault,
                    token_program,
                    vault_program,
                    token_amount,
                    minimum_lp_token_amount,
                )?;

                Ok(())
            },
            &mut ctx.accounts.vault,
            &mut ctx.accounts.vault_lp_mint,
            &mut ctx.accounts.user_lp,
            &mut ctx.accounts.partner,
            &mut ctx.accounts.user,
        )?;
        Ok(())
    }

    /// withdraw
    #[allow(clippy::needless_lifetimes)]
    pub fn withdraw<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DepositWithdrawLiquidity>,
        unmint_amount: u64,
        min_out_amount: u64,
    ) -> Result<()> {
        let partner_key = ctx.accounts.partner.key();
        let owner_key = ctx.accounts.owner.key();
        let user_seeds = &[
            partner_key.as_ref(),
            owner_key.as_ref(),
            &[ctx.accounts.user.bump],
        ];

        let vault = &ctx.accounts.vault.to_account_info();
        let user = &ctx.accounts.user.to_account_info();
        let vault_lp_mint = &ctx.accounts.vault_lp_mint.to_account_info();
        let user_lp = &ctx.accounts.user_lp.to_account_info();

        let user_token = &ctx.accounts.user_token.to_account_info();
        let token_vault = &ctx.accounts.token_vault.to_account_info();
        let token_program = &ctx.accounts.token_program.to_account_info();
        let vault_program = &ctx.accounts.vault_program.to_account_info();
        update_liquidity_wrapper(
            move || {
                VaultUtils::withdraw(
                    vault,
                    vault_lp_mint,
                    user_token,
                    user_lp,
                    user,
                    token_vault,
                    token_program,
                    vault_program,
                    unmint_amount,
                    min_out_amount,
                    &[&user_seeds[..]],
                )?;

                Ok(())
            },
            &mut ctx.accounts.vault,
            &mut ctx.accounts.vault_lp_mint,
            &mut ctx.accounts.user_lp,
            &mut ctx.accounts.partner,
            &mut ctx.accounts.user,
        )?;
        Ok(())
    }

    /// withdraw directly from strategy
    #[allow(clippy::needless_lifetimes)]
    pub fn withdraw_directly_from_strategy<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, WithdrawDirectlyFromStrategy<'info>>,
        unmint_amount: u64,
        min_out_amount: u64,
    ) -> Result<()> {
        let partner_key = ctx.accounts.partner.key();
        let owner_key = ctx.accounts.owner.key();
        let user_seeds = &[
            partner_key.as_ref(),
            owner_key.as_ref(),
            &[ctx.accounts.user.bump],
        ];

        let vault = &ctx.accounts.vault.to_account_info();
        let strategy = &ctx.accounts.strategy.to_account_info();
        let reserve = &ctx.accounts.reserve.to_account_info();
        let strategy_program = &ctx.accounts.strategy_program.to_account_info();
        let collateral_vault = &ctx.accounts.collateral_vault.to_account_info();
        let lp_mint = &ctx.accounts.vault_lp_mint.to_account_info();
        let fee_vault = &ctx.accounts.fee_vault.to_account_info();
        let user_lp = &ctx.accounts.user_lp.to_account_info();

        let user_token = &ctx.accounts.user_token.to_account_info();
        let token_vault = &ctx.accounts.token_vault.to_account_info();
        let token_program = &ctx.accounts.token_program.to_account_info();
        let vault_program = &ctx.accounts.vault_program.to_account_info();
        let user = &ctx.accounts.user.to_account_info();
        let remaining_accounts = ctx.remaining_accounts;
        update_liquidity_wrapper(
            move || {
                VaultUtils::withdraw_directly_from_strategy(
                    vault,
                    strategy,
                    reserve,
                    strategy_program,
                    collateral_vault,
                    token_vault,
                    lp_mint,
                    fee_vault,
                    user_token,
                    user_lp,
                    user,
                    token_program,
                    vault_program,
                    remaining_accounts,
                    unmint_amount,
                    min_out_amount,
                    &[&user_seeds[..]],
                )?;

                Ok(())
            },
            &mut ctx.accounts.vault,
            &mut ctx.accounts.vault_lp_mint,
            &mut ctx.accounts.user_lp,
            &mut ctx.accounts.partner,
            &mut ctx.accounts.user,
        )?;
        Ok(())
    }

    /// fund partner the sharing fee
    pub fn fund_partner<'a, 'b, 'c, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, FundPartner<'info>>,
        amount: u64,
    ) -> Result<()> {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                Transfer {
                    from: ctx.accounts.funder_token.to_account_info(),
                    to: ctx.accounts.partner_token.to_account_info(),
                    authority: ctx.accounts.funder.to_account_info(),
                },
            ),
            amount,
        )?;
        // deduct fee amount, if amount > self.outstanding_fee, then it returns MathOverflow
        let partner = &mut ctx.accounts.partner;
        partner.outstanding_fee = partner
            .outstanding_fee
            .checked_sub(amount)
            .ok_or(VaultError::MathOverflow)?;
        Ok(())
    }
}

/// update liquidity
pub fn update_liquidity_wrapper<'info>(
    update_liquidity_fn: impl FnOnce() -> Result<()>,
    vault: &mut Account<'info, Vault>,
    vault_lp_mint: &mut Account<'info, Mint>,
    user_lp: &mut Account<'info, TokenAccount>,
    partner: &mut Account<'info, Partner>,
    user: &mut Account<'info, User>,
) -> Result<()> {
    // accrue fee
    let current_time = u64::try_from(Clock::get()?.unix_timestamp)
        .ok()
        .ok_or(VaultError::MathOverflow)?;
    let virtual_price = vault
        .get_virtual_price(current_time, vault_lp_mint.supply)
        .ok_or(VaultError::MathOverflow)?;

    let fee = user
        .get_fee(virtual_price, partner.fee_ratio)
        .ok_or(VaultError::MathOverflow)?;

    msg!("fee: {}", fee);
    emit!(PartnerFee { fee });
    // acrrure fee for partner
    partner.accrue_fee(fee).ok_or(VaultError::MathOverflow)?;

    update_liquidity_fn()?;

    // save new user state
    user_lp.reload()?;
    user.set_new_state(virtual_price, user_lp.amount);

    Ok(())
}

/// InitPartner struct
#[derive(Accounts)]
pub struct InitPartner<'info> {
    /// Vault account
    #[account(
            init,
            seeds = [
                vault.key().as_ref(), partner_token.key().as_ref(),
            ],
            bump,
            payer = admin,
            space = 200 // data + buffer,
        )]
    pub partner: Box<Account<'info, Partner>>,
    /// CHECK:
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK: partner_token mint must be same as native token in vault
    #[account(constraint = vault.token_mint == partner_token.mint)]
    pub partner_token: Box<Account<'info, TokenAccount>>,

    /// Admin address
    #[account(mut, constraint = admin.key() == get_admin_address())]
    pub admin: Signer<'info>,

    /// System program account
    pub system_program: Program<'info, System>,
    /// Rent account
    pub rent: Sysvar<'info, Rent>,
    /// Token program account
    pub token_program: Program<'info, Token>,
}

/// UpdateFeeRatio struct
#[derive(Accounts)]
pub struct UpdateFeeRatio<'info> {
    /// Vault account
    #[account(mut)]
    pub partner: Box<Account<'info, Partner>>,

    /// Admin address
    #[account(constraint = admin.key() == get_admin_address())]
    pub admin: Signer<'info>,
}

/// InitUser struct
#[derive(Accounts)]
pub struct InitUser<'info> {
    /// User account
    #[account(
            init,
            seeds = [
                partner.key().as_ref(), owner.key().as_ref(),
            ],
            bump,
            payer = owner,
            space = 200 // data + buffer,
        )]
    pub user: Box<Account<'info, User>>,
    /// CHECK:
    pub partner: Box<Account<'info, Partner>>,

    /// signer address
    #[account(mut)]
    pub owner: Signer<'info>,
    /// System program account
    pub system_program: Program<'info, System>,
    /// Rent account
    pub rent: Sysvar<'info, Rent>,
}

/// Need to check whether we can convert to unchecked account
#[derive(Accounts)]
pub struct DepositWithdrawLiquidity<'info> {
    /// CHECK:
    #[account(mut, has_one = vault)]
    pub partner: Box<Account<'info, Partner>>,
    /// CHECK:
    #[account(mut, has_one = partner, has_one = owner)]
    pub user: Box<Account<'info, User>>,
    /// CHECK:
    pub vault_program: Program<'info, MercurialVault>,
    /// CHECK:
    #[account(mut)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK:
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub vault_lp_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    #[account(mut)]
    pub user_token: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut, constraint = user_lp.owner == user.key())] //mint to account of user PDA
    pub user_lp: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    pub owner: Signer<'info>,
    /// CHECK:
    pub token_program: Program<'info, Token>,
}

/// Accounts for withdraw directly from a strategy
#[derive(Accounts)]
pub struct WithdrawDirectlyFromStrategy<'info> {
    /// CHECK:
    #[account(mut, has_one = vault)]
    pub partner: Box<Account<'info, Partner>>,
    /// CHECK:
    #[account(mut, has_one = partner, has_one = owner)]
    pub user: Box<Account<'info, User>>,
    /// CHECK:
    pub vault_program: Program<'info, MercurialVault>,

    /// vault
    #[account(mut)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK:
    #[account(mut)]
    pub strategy: UncheckedAccount<'info>,

    /// CHECK:: Reserve account
    #[account(mut)]
    pub reserve: UncheckedAccount<'info>,

    /// CHECK:: Strategy program
    pub strategy_program: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub collateral_vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub token_vault: UncheckedAccount<'info>,
    /// lp_mint
    #[account(mut)]
    pub vault_lp_mint: Box<Account<'info, Mint>>,
    /// CHECK:
    #[account(mut)]
    pub fee_vault: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub user_token: UncheckedAccount<'info>,
    /// user_lp
    #[account(mut, constraint = user_lp.owner == user.key())] //unmint from account of user PDA
    pub user_lp: Box<Account<'info, TokenAccount>>,
    /// user
    pub owner: Signer<'info>,
    /// token_program
    pub token_program: Program<'info, Token>,
}

/// Need to check whether we can convert to unchecked account
#[derive(Accounts)]
pub struct FundPartner<'info> {
    /// CHECK:
    #[account(mut, has_one = partner_token)]
    pub partner: Box<Account<'info, Partner>>,
    /// CHECK:
    #[account(mut)]
    pub partner_token: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    #[account(mut, constraint = funder_token.key() != partner_token.key() @ VaultError::WrongFunderToken)]
    pub funder_token: Box<Account<'info, TokenAccount>>,
    /// CHECK:
    pub funder: Signer<'info>,
    /// CHECK:
    pub token_program: Program<'info, Token>,
}

/// Partner struct
#[account]
#[derive(Debug)]
pub struct Partner {
    /// partner token address, which is used to get fee later (fee is in native token)
    pub partner_token: Pubkey, // 32
    /// vault address that partner integrates
    pub vault: Pubkey, // 32
    /// total fee that partner get, but haven't sent yet
    pub outstanding_fee: u64, // 8
    /// fee ratio partner get in performance fee
    pub fee_ratio: u64, // 8
    // cumulative fee partner get from start
    pub cumulative_fee: u128, // 16
}

impl Partner {
    /// accrue fee
    pub fn accrue_fee(&mut self, fee: u64) -> Option<()> {
        self.outstanding_fee = self.outstanding_fee.checked_add(fee)?;
        let max = u128::MAX;
        let buffer = max - self.cumulative_fee;
        let fee: u128 = fee.into();
        if buffer >= fee {
            // only add if we have enough room
            self.cumulative_fee = self.cumulative_fee.checked_add(fee)?;
        }
        Some(())
    }
}

/// User struct
#[account]
#[derive(Default, Debug)]
pub struct User {
    owner: Pubkey,
    /// partner address, each user can integrate with more partners
    partner: Pubkey,
    /// current virtual price
    current_virtual_price: u64,
    /// lp_token that user holds
    lp_token: u64,
    /// user bump
    bump: u8,
}

impl User {
    /// get fee per user
    pub fn get_fee(&mut self, virtual_price: u64, fee_ratio: u64) -> Option<u64> {
        if virtual_price <= self.current_virtual_price {
            // if virtual price is reduced, then no fee is accrued
            return Some(0);
        }
        let yield_earned = u128::from(self.lp_token)
            .checked_mul(u128::from(
                virtual_price.checked_sub(self.current_virtual_price)?,
            ))?
            .checked_div(PRICE_PRECISION)?;

        let performance_fee_by_vault = yield_earned
            .checked_mul(PERFORMANCE_FEE_NUMERATOR)?
            .checked_div(PERFORMANCE_FEE_DENOMINATOR)?;

        let fee_sharing = u64::try_from(
            performance_fee_by_vault
                .checked_mul(fee_ratio.into())?
                .checked_div(FEE_DENOMINATOR)?,
        )
        .ok()?;

        Some(fee_sharing)
    }

    /// set new state
    pub fn set_new_state(&mut self, virtual_price: u64, lp_token: u64) {
        self.current_virtual_price = virtual_price;
        self.lp_token = lp_token;
    }
}

/// VaultError struct
#[error_code]
pub enum VaultError {
    /// MathOverflow
    #[msg("Math operation overflow")]
    MathOverflow,

    /// InvalidOwner
    #[msg("Invalid owner")]
    InvalidOwner,

    /// InvalidFeeRatio
    #[msg("Invalid ratio")]
    InvalidFeeRatio,

    #[msg("Funder token account must be different from partner token account")]
    WrongFunderToken,
}

#[event]
/// PartnerFee struct
pub struct PartnerFee {
    fee: u64,
}
