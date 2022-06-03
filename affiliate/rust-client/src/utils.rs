use anchor_client::solana_client::rpc_response::RpcSimulateTransactionResult;
use anchor_client::RequestBuilder;
use anchor_client::{
    solana_client::rpc_response::Response,
    solana_sdk::{signature::Signer, transaction::Transaction},
    Program,
};
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::keypair::read_keypair_file;

pub fn parse_event_log<
    T: anchor_lang::AnchorDeserialize + anchor_lang::AnchorSerialize + anchor_lang::Discriminator,
>(
    logs: &Vec<String>,
) -> Option<T> {
    for log in logs.into_iter() {
        if log.starts_with("Program data:") {
            // Skip the prefix "Program data: "
            // Event logged has been changed to Program data: instead of Program log:
            // https://github.com/project-serum/anchor/pull/1608/files
            let log_info: String = log.chars().skip(14).collect();
            let log_buf = anchor_lang::__private::base64::decode(log_info.as_bytes());
            if log_buf.is_ok() {
                let log_buf = log_buf.unwrap();
                // Check for event discriminator, it is a 8-byte prefix
                if log_buf[0..8] == T::discriminator() {
                    // Skip event discriminator when deserialize
                    return T::try_from_slice(&log_buf[8..]).ok();
                }
            }
        }
    }
    None
}

pub fn simulate_transaction(
    builder: &RequestBuilder,
    program: &Program,
    signers: &Vec<&dyn Signer>,
) -> Result<Response<RpcSimulateTransactionResult>, Box<dyn std::error::Error>> {
    let instructions = builder.instructions()?;
    let rpc_client = program.rpc();
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&program.payer()),
        signers,
        recent_blockhash,
    );
    let simulation = rpc_client.simulate_transaction(&tx)?;
    Ok(simulation)
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

pub fn default_keypair() -> Keypair {
    read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
        .expect("Requires a keypair file")
}
