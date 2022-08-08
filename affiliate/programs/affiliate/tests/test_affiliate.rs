#![cfg(feature = "test-bpf")]

mod helpers;
use crate::helpers::affiliate::*;
use crate::helpers::utils::*;
use affiliate::vault_utils::PRICE_PRECISION;
use helpers::vault::add_vault;
use helpers::vault::AddVaultRes;
use mercurial_vault::PERFORMANCE_FEE_DENOMINATOR;
use mercurial_vault::PERFORMANCE_FEE_NUMERATOR;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

struct SetupContext {
    vault: AddVaultRes,
    decimal: u8,
    context: ProgramTestContext,
    token_mint: AddMintRes,
}

async fn setup_prerequisite() -> SetupContext {
    let affiliate_admin = get_admin_keypair();
    let mut test = ProgramTest::new("affiliate", affiliate::id(), processor!(affiliate::entry));
    test.add_program(
        "mercurial_vault",
        mercurial_vault::id(),
        processor!(mercurial_vault::entry),
    );

    let token_decimal = 6;
    let token_mint = add_mint(&mut test, token_decimal);

    let vault = add_vault(&mut test, token_mint.mint_key, token_decimal);
    let mut context: ProgramTestContext = test.start_with_context().await;
    {
        let banks_client = &mut context.banks_client;
        let payer = &mut context.payer;

        // Needed sol to init partner
        transfer_native_sol(
            banks_client,
            &payer,
            affiliate_admin.pubkey(),
            10_000_000_000,
        )
        .await;
    }

    SetupContext {
        context,
        decimal: token_decimal,
        vault,
        token_mint,
    }
}

#[tokio::test]
pub async fn test_update_fee_ratio() {
    let mut setup_context = setup_prerequisite().await;
    let mint_info = &setup_context.token_mint;
    let usdc_vault = setup_context.vault;
    let partner_wallet = Keypair::new();
    let mut context = setup_context.context;

    let user_wallet = Keypair::new();
    transfer_native_sol(
        &mut context.banks_client,
        &context.payer,
        user_wallet.pubkey(),
        10_000_000_000,
    )
    .await;

    let partner_pda = init_partner(
        &mut context.banks_client,
        usdc_vault,
        partner_wallet.pubkey(),
        &context.payer,
    )
    .await;
    let user_pda = init_user(
        &mut context.banks_client,
        &user_wallet,
        partner_pda,
        &context.payer,
    )
    .await;

    mint_to(
        &mut context.banks_client,
        &mint_info,
        100_000_000_000,
        user_wallet.pubkey(),
        &context.payer,
    )
    .await;

    deposit(
        &mut context.banks_client,
        usdc_vault,
        &user_wallet,
        500_000_000,
        partner_pda,
        user_pda,
        &context.payer,
    )
    .await;

    let interest_amount = 10_000_000;
    simulate_interest(&mut context, usdc_vault, interest_amount).await;
    let performance_fee =
        interest_amount as u128 * PERFORMANCE_FEE_NUMERATOR / PERFORMANCE_FEE_DENOMINATOR;
    let partner_fee = performance_fee * 5_000 / 10_000; // 50%

    deposit(
        &mut context.banks_client,
        usdc_vault,
        &user_wallet,
        500_000_000,
        partner_pda,
        user_pda,
        &context.payer,
    )
    .await;
    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.fee_ratio, 5_000);
    assert_eq!(partner_state.cumulative_fee as u128, partner_fee);

    update_fee_ratio(
        &mut context.banks_client,
        partner_pda,
        3_000, // 30%
        &context.payer,
    )
    .await;
    simulate_interest(&mut context, usdc_vault, interest_amount).await;
    let performance_fee =
        interest_amount as u128 * PERFORMANCE_FEE_NUMERATOR / PERFORMANCE_FEE_DENOMINATOR;
    let new_partner_fee = performance_fee * 3_000 / 10_000; // 30%

    deposit(
        &mut context.banks_client,
        usdc_vault,
        &user_wallet,
        500_000_000,
        partner_pda,
        user_pda,
        &context.payer,
    )
    .await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.fee_ratio, 3_000);
    assert_eq!(
        partner_state.cumulative_fee as u128 - partner_fee,
        new_partner_fee
    );
}

#[tokio::test]
pub async fn test_init_partner() {
    let mut setup_context = setup_prerequisite().await;
    let banks_client = &mut setup_context.context.banks_client;
    let payer = &setup_context.context.payer;
    let usdc_vault = setup_context.vault;
    let partner_wallet = Keypair::new();

    let partner_pda = init_partner(banks_client, usdc_vault, partner_wallet.pubkey(), payer).await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, banks_client).await;

    let partner_token = spl_associated_token_account::get_associated_token_address(
        &partner_wallet.pubkey(),
        &usdc_vault.token_mint_pubkey,
    );

    assert_eq!(partner_state.cumulative_fee, 0);
    assert_eq!(partner_state.fee_ratio, 5_000); // Default fee ratio
    assert_eq!(partner_state.outstanding_fee, 0);
    assert_eq!(partner_state.partner_token, partner_token);
    assert_eq!(partner_state.user_count, 0);
    assert_eq!(partner_state.vault, usdc_vault.vault_pubkey);
}

#[tokio::test]
async fn test_init_user() {
    let mut setup_context = setup_prerequisite().await;
    let banks_client = &mut setup_context.context.banks_client;
    let payer = &setup_context.context.payer;
    let usdc_vault = setup_context.vault;
    let partner_wallet = Keypair::new();

    let user_wallet = Keypair::new();
    transfer_native_sol(banks_client, &payer, user_wallet.pubkey(), 10_000_000_000).await;

    let partner_pda = init_partner(banks_client, usdc_vault, partner_wallet.pubkey(), payer).await;
    init_user(banks_client, &user_wallet, partner_pda, payer).await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, banks_client).await;

    assert_eq!(partner_state.user_count, 1u64);

    let user_wallet = Keypair::new();
    transfer_native_sol(banks_client, &payer, user_wallet.pubkey(), 10_000_000_000).await;
    let user_pda = init_user(banks_client, &user_wallet, partner_pda, payer).await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, banks_client).await;

    assert_eq!(partner_state.user_count, 2u64);

    let user_state = get_anchor_seder_account::<affiliate::User>(&user_pda, banks_client).await;

    assert_eq!(user_state.current_virtual_price, 0);
    assert_eq!(user_state.partner, partner_pda);
    assert_eq!(user_state.lp_token, 0);
    assert_eq!(user_state.owner, user_wallet.pubkey());
}

#[tokio::test]
async fn test_metrics() {
    let mut setup_context = setup_prerequisite().await;
    let usdc_vault = setup_context.vault;
    let partner_wallet = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();
    let mint_info = &setup_context.token_mint;

    let mut context = &mut setup_context.context;
    let partner_pda = init_partner(
        &mut context.banks_client,
        usdc_vault,
        partner_wallet.pubkey(),
        &context.payer,
    )
    .await;

    transfer_native_sol(
        &mut context.banks_client,
        &context.payer,
        alice.pubkey(),
        10_000_000_000,
    )
    .await;
    transfer_native_sol(
        &mut context.banks_client,
        &context.payer,
        bob.pubkey(),
        10_000_000_000,
    )
    .await;

    let alice_user_pda = init_user(
        &mut context.banks_client,
        &alice,
        partner_pda,
        &context.payer,
    )
    .await;

    let alice_user_state =
        get_anchor_seder_account::<affiliate::User>(&alice_user_pda, &mut context.banks_client)
            .await;
    assert_eq!(alice_user_state.owner, alice.pubkey());
    assert_eq!(alice_user_state.partner, partner_pda);
    assert_eq!(alice_user_state.current_virtual_price, 0);
    assert_eq!(alice_user_state.lp_token, 0);

    let bob_user_pda =
        init_user(&mut context.banks_client, &bob, partner_pda, &context.payer).await;

    let bob_user_state =
        get_anchor_seder_account::<affiliate::User>(&bob_user_pda, &mut context.banks_client).await;
    assert_eq!(bob_user_state.owner, bob.pubkey());
    assert_eq!(bob_user_state.partner, partner_pda);
    assert_eq!(bob_user_state.current_virtual_price, 0);
    assert_eq!(bob_user_state.lp_token, 0);

    mint_to(
        &mut context.banks_client,
        &mint_info,
        100_000_000_000,
        alice.pubkey(),
        &context.payer,
    )
    .await;

    mint_to(
        &mut context.banks_client,
        &mint_info,
        100_000_000_000,
        bob.pubkey(),
        &context.payer,
    )
    .await;

    let alice_lp = deposit(
        &mut context.banks_client,
        usdc_vault,
        &alice,
        500_000_000,
        partner_pda,
        alice_user_pda,
        &context.payer,
    )
    .await;

    let alice_lp_state = get_anchor_seder_account::<anchor_spl::token::TokenAccount>(
        &alice_lp,
        &mut context.banks_client,
    )
    .await;

    let alice_user_state =
        get_anchor_seder_account::<affiliate::User>(&alice_user_pda, &mut context.banks_client)
            .await;
    assert_eq!(alice_user_state.owner, alice.pubkey());
    assert_eq!(alice_user_state.partner, partner_pda);
    assert_eq!(
        alice_user_state.current_virtual_price as u128,
        PRICE_PRECISION
    );
    assert_eq!(alice_user_state.lp_token, alice_lp_state.amount);

    let bob_lp = deposit(
        &mut context.banks_client,
        usdc_vault,
        &bob,
        500_000_000,
        partner_pda,
        bob_user_pda,
        &context.payer,
    )
    .await;

    let bob_lp_state = get_anchor_seder_account::<anchor_spl::token::TokenAccount>(
        &bob_lp,
        &mut context.banks_client,
    )
    .await;

    let bob_user_state =
        get_anchor_seder_account::<affiliate::User>(&bob_user_pda, &mut context.banks_client).await;
    assert_eq!(bob_user_state.owner, bob.pubkey());
    assert_eq!(bob_user_state.partner, partner_pda);
    assert_eq!(
        bob_user_state.current_virtual_price as u128,
        PRICE_PRECISION
    );
    assert_eq!(bob_user_state.lp_token, bob_lp_state.amount);

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.liquidity, 1_000_000_000);
    assert_eq!(partner_state.user_count, 2);
    assert_eq!(partner_state.outstanding_fee, 0);

    let interest_amount = 10_000_000;
    simulate_interest(&mut context, usdc_vault, interest_amount).await;
    let performance_fee =
        interest_amount as u128 * PERFORMANCE_FEE_NUMERATOR / PERFORMANCE_FEE_DENOMINATOR;
    let total_partner_fee = performance_fee * 5_000 / 10_000; // 50%
                                                              // Alice and bob deposited equal amount
    let bob_contributed_fee = total_partner_fee / 2;
    let alice_contributed_fee = total_partner_fee / 2;

    deposit(
        &mut context.banks_client,
        usdc_vault,
        &bob,
        500_000_000,
        partner_pda,
        bob_user_pda,
        &context.payer,
    )
    .await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.liquidity > 1_500_000_000, true); // principal (alice 500_000_000 + bob 1_000_000_000) + interest (5_000_000)
    assert_eq!(partner_state.outstanding_fee as u128, bob_contributed_fee);

    deposit(
        &mut context.banks_client,
        usdc_vault,
        &alice,
        500_000_000,
        partner_pda,
        alice_user_pda,
        &context.payer,
    )
    .await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.liquidity > 2_000_000_000, true); // principal (alice 1_000_000_000 + bob 1_000_000_000) + interest (bob 5_000_000 + alice 5_000_000)
    assert_eq!(
        partner_state.outstanding_fee as u128,
        bob_contributed_fee + alice_contributed_fee
    );

    let bob_lp_state = get_anchor_seder_account::<anchor_spl::token::TokenAccount>(
        &bob_lp,
        &mut context.banks_client,
    )
    .await;

    withdraw(
        &mut context.banks_client,
        usdc_vault,
        &bob,
        bob_lp_state.amount,
        partner_pda,
        bob_user_pda,
        &bob,
    )
    .await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.liquidity > 1_000_000_000, true); // principal (alice 1_000_000_000) + interest (alice 5_000_000)

    let alice_lp_state = get_anchor_seder_account::<anchor_spl::token::TokenAccount>(
        &alice_lp,
        &mut context.banks_client,
    )
    .await;

    withdraw(
        &mut context.banks_client,
        usdc_vault,
        &alice,
        alice_lp_state.amount,
        partner_pda,
        alice_user_pda,
        &alice,
    )
    .await;

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, &mut context.banks_client)
            .await;
    assert_eq!(partner_state.liquidity, 0);
}

#[tokio::test]
async fn test_deposit() {
    let mut setup_context = setup_prerequisite().await;
    let banks_client = &mut setup_context.context.banks_client;
    let payer = &setup_context.context.payer;
    let usdc_vault = setup_context.vault;
    let partner_wallet = Keypair::new();
    let mint_info = &setup_context.token_mint;

    let partner_pda = init_partner(banks_client, usdc_vault, partner_wallet.pubkey(), payer).await;

    let user_wallet = Keypair::new();
    transfer_native_sol(banks_client, &payer, user_wallet.pubkey(), 10_000_000_000).await;
    let user_pda = init_user(banks_client, &user_wallet, partner_pda, payer).await;

    mint_to(
        banks_client,
        &mint_info,
        1_000_000_000,
        user_wallet.pubkey(),
        payer,
    )
    .await;

    let user_lp = deposit(
        banks_client,
        usdc_vault,
        &user_wallet,
        500_000_000,
        partner_pda,
        user_pda,
        payer,
    )
    .await;

    let user_lp_state =
        get_anchor_seder_account::<anchor_spl::token::TokenAccount>(&user_lp, banks_client).await;
    assert_eq!(user_lp_state.amount, 500_000_000);

    let user_state = get_anchor_seder_account::<affiliate::User>(&user_pda, banks_client).await;
    assert_eq!(user_state.owner, user_wallet.pubkey());
    assert_eq!(user_state.partner, partner_pda);
    assert_eq!(user_state.current_virtual_price as u128, PRICE_PRECISION); // virtual price = 1
    assert_eq!(user_state.lp_token, user_lp_state.amount);

    let partner_state =
        get_anchor_seder_account::<affiliate::Partner>(&partner_pda, banks_client).await;
    println!("{:#?}", partner_state);
    assert_eq!(partner_state.liquidity, 500_000_000);
}
