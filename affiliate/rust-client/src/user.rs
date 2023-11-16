use anyhow::Result;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::system_instruction;
use solana_sdk::system_program;
use spl_associated_token_account;
use std::str::FromStr;

pub fn deposit(
    program_client: &anchor_client::Program,
    token_mint: Pubkey,
    base: Pubkey,
    partner: String,
    token_amount: u64,
) -> Result<()> {
    println!("deposit {} partner {}", token_amount, partner);
    let partner = Pubkey::from_str(&partner).unwrap();

    let (vault, _vault_bump) = Pubkey::find_program_address(
        &[b"vault".as_ref(), token_mint.as_ref(), base.as_ref()],
        &mercurial_vault::id(),
    );

    let (token_vault, _token_vault_bump) = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), vault.as_ref()],
        &mercurial_vault::id(),
    );

    let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
    let lp_mint = vault_state.lp_mint;

    let user_token = get_or_create_ata(program_client, token_mint, program_client.payer())?;

    let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner)?;
    let (user, _nonce) = Pubkey::find_program_address(
        &[partner.as_ref(), program_client.payer().as_ref()],
        &affiliate::id(),
    );
    // check whether user is existed
    let rpc_client = program_client.rpc();
    if rpc_client.get_account_data(&user).is_err() {
        // create user account
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

        let signature = builder.send()?;
        println!("create user {}", signature);
    }

    let user_lp = get_or_create_ata(program_client, lp_mint, user)?;

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::DepositWithdrawLiquidity {
            partner,
            user,
            vault: vault,
            vault_program: mercurial_vault::id(),
            token_vault: token_vault,
            vault_lp_mint: lp_mint,
            user_token,
            user_lp,
            owner: program_client.payer(),
            token_program: spl_token::id(),
        })
        .args(mercurial_vault::instruction::Deposit {
            token_amount,
            minimum_lp_token_amount: 0,
        });

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}

pub fn withdraw(
    program_client: &anchor_client::Program,
    token_mint: Pubkey,
    base: Pubkey,
    partner: String,
    unmint_amount: u64,
) -> Result<()> {
    println!("withdraw {} lp token partner {}", unmint_amount, partner);
    let partner = Pubkey::from_str(&partner).unwrap();
    let (vault, _vault_bump) = Pubkey::find_program_address(
        &[b"vault".as_ref(), token_mint.as_ref(), base.as_ref()],
        &mercurial_vault::id(),
    );

    let (token_vault, _token_vault_bump) = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), vault.as_ref()],
        &mercurial_vault::id(),
    );

    let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
    let lp_mint = vault_state.lp_mint;

    let user_token = get_or_create_ata(program_client, token_mint, program_client.payer())?;

    let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
    println!(
        "withdraw {} lp token partner {} {}",
        unmint_amount, partner, partner_token
    );
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());
    // check whether partner is existed
    let _partner_state: affiliate::Partner = program_client.account(partner)?;
    let (user, _nonce) = Pubkey::find_program_address(
        &[partner.as_ref(), program_client.payer().as_ref()],
        &affiliate::id(),
    );
    let user_lp = get_or_create_ata(program_client, lp_mint, user)?;
    // check whether user is existed
    let rpc_client = program_client.rpc();
    if rpc_client.get_account_data(&user).is_err() {
        // create user account
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

        let signature = builder.send()?;
        println!("create user {}", signature);
    }

    let builder = program_client
        .request()
        .accounts(affiliate::accounts::DepositWithdrawLiquidity {
            partner,
            user,
            vault: vault,
            vault_program: mercurial_vault::id(),
            token_vault: token_vault,
            vault_lp_mint: lp_mint,
            user_token,
            user_lp,
            owner: program_client.payer(),
            token_program: spl_token::id(),
        })
        .args(affiliate::instruction::Withdraw {
            unmint_amount,
            min_out_amount: 0,
        });

    let signature = builder.send()?;
    println!("{}", signature);

    Ok(())
}

pub fn view_user(
    program_client: &anchor_client::Program,
    vault: Pubkey,
    partner: String,
) -> Result<()> {
    let partner = Pubkey::from_str(&partner).unwrap();
    let vault_state: mercurial_vault::state::Vault = program_client.account(vault)?;
    let token_mint = vault_state.token_mint;
    let partner_token = get_or_create_ata(program_client, token_mint, partner)?;
    let (partner, _nonce) =
        Pubkey::find_program_address(&[vault.as_ref(), partner_token.as_ref()], &affiliate::id());

    let (user, _nonce) = Pubkey::find_program_address(
        &[partner.as_ref(), program_client.payer().as_ref()],
        &affiliate::id(),
    );
    // check whether user is existed
    let user_state: affiliate::User = program_client.account(user)?;
    println!("{:?}", user_state);

    Ok(())
}

pub fn get_or_create_ata(
    program_client: &anchor_client::Program,
    token_mint: Pubkey,
    user: Pubkey,
) -> Result<Pubkey> {
    let user_token_account =
        spl_associated_token_account::get_associated_token_address(&user, &token_mint);
    let rpc_client = program_client.rpc();
    if rpc_client.get_account_data(&user_token_account).is_err() {
        println!("Create ATA for TOKEN {} \n", &token_mint);

        let builder = program_client.request().instruction(
            spl_associated_token_account::create_associated_token_account(
                &program_client.payer(),
                &user,
                &token_mint,
            ),
        );

        let signature = builder.send()?;
        println!("{}", signature);
    }
    Ok(user_token_account)
}

pub fn create_mint(
    program_client: &anchor_client::Program,
    mint_keypair: &Keypair,
    authority: Pubkey,
    decimals: u8,
) -> Result<()> {
    let rpc = program_client.rpc();

    let token_mint_account_rent =
        rpc.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;

    let instructions = vec![
        system_instruction::create_account(
            &program_client.payer(),
            &mint_keypair.pubkey(),
            token_mint_account_rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_keypair.pubkey(),
            &authority,
            None,
            decimals,
        )
        .unwrap(),
    ];

    let builder = program_client.request();
    let builder = builder.signer(mint_keypair);
    let builder = instructions
        .into_iter()
        .fold(builder, |bld, ix| bld.instruction(ix));
    let signature = builder.send()?;
    println!("{}", signature);
    Ok(())
}
