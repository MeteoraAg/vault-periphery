use crate::strategy_handler::base::StrategyHandler;
use anyhow::Result;

use solana_program::pubkey::Pubkey;
pub struct MangoHandler {}

impl StrategyHandler for MangoHandler {
    fn withdraw_directly_from_strategy(
        &self,
        _program_client: &anchor_client::Program,
        _strategy: Pubkey,
        _token_mint: Pubkey,
        _base: Pubkey,
        _partner: String,
        _amount: u64,
    ) -> Result<()> {
        panic!("mango is deprecated")
    }
}
