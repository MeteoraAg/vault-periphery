//! Affilicate program
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::integer_arithmetic)]
#![warn(missing_docs)]

use anchor_lang::prelude::*;
pub mod vault_utils;
use crate::vault_utils::{MercurialVault, VaultUtils, Virtualprice, PRICE_PRECISION};
use anchor_spl::token::{Mint, Token, TokenAccount};
use mercurial_vault::state::Vault;
use std::str::FromStr;
use vipers::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// Admin address, only admin can initialize a partner
pub fn get_admin_address() -> Pubkey {
    Pubkey::from_str("DHLXnJdACTY83yKwnUkeoDjqi4QBbsYGa1v8tJL76ViX")
        .expect("Must be correct Solana address")
}

#[program]
pub mod affiliate {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    /// function can be only called by admin
    pub fn init_partner(ctx: Context<InitPartner>) -> Result<()> {
        let partner = &mut ctx.accounts.partner;
        partner.vault = ctx.accounts.vault.key();
        partner.partner_token = ctx.accounts.partner_token.key();
        Ok(())
    }

    /// function can be only called by user
    pub fn init_user(ctx: Context<InitUser>) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.partner = ctx.accounts.partner.key();
        user.owner = ctx.accounts.signer.key();
        user.bump = unwrap_bump!(ctx, "user");
        Ok(())
    }

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
        let vault_lp_mint = &ctx.accounts.vault_lp_mint.to_account_info();
        let user_lp = &ctx.accounts.user_lp.to_account_info();

        let user_token = &ctx.accounts.user_token.to_account_info();
        let token_vault = &ctx.accounts.token_vault.to_account_info();
        let token_program = &ctx.accounts.token_program.to_account_info();
        let vault_program = &ctx.accounts.vault_program.to_account_info();
        let owner = &ctx.accounts.owner.to_account_info();
        update_liquidity_wrapper(
            move || {
                VaultUtils::withdraw(
                    vault,
                    vault_lp_mint,
                    user_token,
                    user_lp,
                    owner,
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
        .get_fee(virtual_price, user_lp.amount)
        .ok_or(VaultError::MathOverflow)?;

    // acrrure fee for partner
    partner.accrue_fee(fee).ok_or(VaultError::MathOverflow)?;

    update_liquidity_fn()?;

    // save new user state
    user_lp.reload()?;
    user.set_new_state(virtual_price, user_lp.amount);

    Ok(())
}

/// Initialize struct
#[derive(Accounts)]
pub struct Initialize {}

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

    pub vault: Box<Account<'info, Vault>>,

    #[account(constraint = vault.lp_mint == partner_token.mint)]
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

/// InitUser struct
#[derive(Accounts)]
pub struct InitUser<'info> {
    /// User account
    #[account(
            init,
            seeds = [
                partner.key().as_ref(), signer.key().as_ref(),
            ],
            bump,
            payer = signer,
            space = 200 // data + buffer,
        )]
    pub user: Box<Account<'info, User>>,

    pub partner: Box<Account<'info, Partner>>,

    /// signer address
    #[account(mut)]
    pub signer: Signer<'info>,
    /// System program account
    pub system_program: Program<'info, System>,
    /// Rent account
    pub rent: Sysvar<'info, Rent>,
}

/// Need to check whether we can convert to unchecked account
#[derive(Accounts)]
pub struct DepositWithdrawLiquidity<'info> {
    #[account(mut)]
    pub vault: Box<Account<'info, Vault>>,

    #[account(mut, has_one = vault)]
    pub partner: Box<Account<'info, Partner>>,

    #[account(mut, has_one = partner, has_one = owner)]
    pub user: Box<Account<'info, User>>,

    pub vault_program: Program<'info, MercurialVault>,

    #[account(mut)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub vault_lp_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub user_token: Box<Account<'info, TokenAccount>>,

    #[account(mut, has_one = owner)]
    pub user_lp: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Partner struct
#[account]
#[derive(Default, Debug)]
pub struct Partner {
    /// partner token address, which is used to get fee later (fee is in lp token)
    partner_token: Pubkey, // 32
    /// vault address that partner integrates
    vault: Pubkey, // 32
    /// total fee that partner get
    total_fee: u64, // 8
}

impl Partner {
    pub fn accrue_fee(&mut self, fee: u64) -> Option<()> {
        self.total_fee = self.total_fee.checked_add(fee)?;
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
    /// lp_tokenthat user holds
    lp_token: u64,
    /// user bump
    bump: u8,
}

impl User {
    pub fn get_fee(&mut self, virtual_price: u64, lp_amount: u64) -> Option<u64> {
        if virtual_price <= self.current_virtual_price {
            // if virtual price is reduced, then no fee is accrued
            return Some(0);
        }
        let fee = u64::try_from(
            u128::from(self.lp_token)
                .checked_mul(u128::from(
                    virtual_price.checked_sub(self.current_virtual_price)?,
                ))?
                .checked_div(virtual_price.into())?
                .checked_div(PRICE_PRECISION)?
                .checked_div(5u128)?, // partner get 20*
        )
        .ok()?;

        Some(fee)
    }
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
}
