use anchor_lang::solana_program::{instruction::Instruction, program_pack::Pack, pubkey::Pubkey};
use assert_matches::assert_matches;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::{
    account::Account,
    native_token::LAMPORTS_PER_SOL,
    program_option::COption,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::{AccountState, Mint};

#[derive(Debug)]
pub struct AddMintRes {
    pub mint_key: Pubkey,
    pub mint_authority: Keypair,
}

pub trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}

pub fn get_admin_keypair() -> Keypair {
    Keypair::from_bytes(&[
        92, 204, 194, 230, 101, 253, 145, 128, 195, 0, 205, 220, 194, 95, 204, 125, 142, 252, 202,
        189, 22, 214, 194, 109, 36, 119, 242, 16, 14, 110, 3, 223, 157, 180, 184, 64, 229, 176, 72,
        100, 194, 1, 8, 96, 229, 236, 37, 1, 26, 123, 62, 192, 34, 174, 29, 7, 247, 106, 209, 47,
        123, 30, 140, 182,
    ])
    .unwrap()
}

pub async fn get_or_create_ata(
    payer: &Keypair,
    token_mint: &Pubkey,
    authority: &Pubkey,
    banks_client: &mut BanksClient,
) -> Pubkey {
    let ata_address = get_associated_token_address(authority, token_mint);
    let ata_account = banks_client.get_account(ata_address).await.unwrap();
    if let None = ata_account {
        create_associated_token_account(payer, token_mint, authority, banks_client).await;
    }
    ata_address
}

pub fn add_mint(test: &mut ProgramTest, decimals: u8) -> AddMintRes {
    let mint_authority = Keypair::new();
    let token_mint_pubkey = Pubkey::new_unique();
    test.add_packable_account(
        token_mint_pubkey,
        10 * LAMPORTS_PER_SOL,
        &Mint {
            is_initialized: true,
            mint_authority: COption::Some(mint_authority.pubkey()),
            decimals,
            supply: 0,
            ..Mint::default()
        },
        &spl_token::id(),
    );
    AddMintRes {
        mint_authority,
        mint_key: token_mint_pubkey,
    }
}

pub async fn transfer_native_sol(
    banks_client: &mut BanksClient,
    from: &Keypair,
    destination: Pubkey,
    amount: u64,
) {
    let ix = system_instruction::transfer(&from.pubkey(), &destination, amount);
    process_and_assert_ok(&[ix], &from, &[&from], banks_client).await;
}

pub fn add_associated_token_account(
    test: &mut ProgramTest,
    owner: Pubkey,
    amount: u64,
    token_mint: Pubkey,
) -> Pubkey {
    let ata = get_associated_token_address(&owner, &token_mint);
    test.add_packable_account(
        ata,
        10 * LAMPORTS_PER_SOL,
        &spl_token::state::Account {
            mint: token_mint,
            owner,
            state: AccountState::Initialized,
            amount,
            ..spl_token::state::Account::default()
        },
        &spl_token::id(),
    );
    ata
}

pub async fn transfer_token(
    amount: u64,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    signer_pubkey: &Pubkey,
    signer: &Keypair,
    payer: &Keypair,
    banks_client: &mut BanksClient,
) {
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        source_pubkey,
        destination_pubkey,
        signer_pubkey,
        &[signer_pubkey],
        amount,
    )
    .unwrap();
    process_and_assert_ok(&[ix], payer, &[signer], banks_client).await;
}

pub async fn mint_to(
    banks_client: &mut BanksClient,
    mint: &AddMintRes,
    amount: u64,
    destination_wallet: Pubkey,
    payer: &Keypair,
) {
    let destination_ata =
        get_or_create_ata(payer, &mint.mint_key, &destination_wallet, banks_client).await;
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint.mint_key,
        &destination_ata,
        &mint.mint_authority.pubkey(),
        &[&mint.mint_authority.pubkey()],
        amount,
    )
    .unwrap();
    process_and_assert_ok(&[ix], &payer, &[&payer, &mint.mint_authority], banks_client).await;
}

pub async fn get_packable_seder_account<T: Pack>(
    account: &Pubkey,
    banks_client: &mut BanksClient,
) -> T {
    let account = banks_client.get_account(*account).await.unwrap().unwrap();
    T::unpack_from_slice(&account.data.as_slice()).unwrap()
}

pub async fn create_associated_token_account(
    payer: &Keypair,
    token_mint: &Pubkey,
    authority: &Pubkey,
    banks_client: &mut BanksClient,
) {
    let ins = vec![
        spl_associated_token_account::create_associated_token_account(
            &payer.pubkey(),
            &authority,
            &token_mint,
        ),
    ];

    process_and_assert_ok(&ins, payer, &[payer], banks_client).await;
}

pub async fn get_anchor_seder_account<T: anchor_lang::AccountDeserialize>(
    account: &Pubkey,
    banks_client: &mut BanksClient,
) -> T {
    let account = banks_client.get_account(*account).await.unwrap().unwrap();
    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}

pub async fn process_and_assert_err(
    instructions: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
    banks_client: &mut BanksClient,
) {
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    let mut all_signers = vec![payer];
    all_signers.extend_from_slice(signers);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &all_signers,
        recent_blockhash,
    );
    match banks_client.process_transaction(tx).await {
        Ok(()) => panic!(),
        _ => {}
    }
}

pub async fn process_and_assert_ok(
    instructions: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
    banks_client: &mut BanksClient,
) {
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();

    let mut all_signers = vec![payer];
    all_signers.extend_from_slice(signers);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &all_signers,
        recent_blockhash,
    );

    assert_matches!(banks_client.process_transaction(tx).await, Ok(()));
}
