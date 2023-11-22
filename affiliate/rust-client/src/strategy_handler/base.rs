use crate::strategy_handler::port_finance_without_lm::PortFinanceWithoutLMHandler;
use crate::strategy_handler::solend_with_lm::SolendWithLMHandler;
use crate::strategy_handler::solend_without_lm::SolendWithoutLMHandler;
use anchor_client::Cluster;
use anyhow::Result;
use async_trait::async_trait;
use mercurial_vault::strategy::base::StrategyType;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;

pub fn get_strategy_handler(strategy_type: StrategyType) -> Box<dyn StrategyHandler> {
    match strategy_type {
        StrategyType::PortFinanceWithoutLM => Box::new(PortFinanceWithoutLMHandler {}),
        StrategyType::PortFinanceWithLM => panic!("Protocol is not supported"),
        StrategyType::SolendWithoutLM => Box::new(SolendWithoutLMHandler {}),
        StrategyType::Mango => panic!("Protocol is not supported"),
        StrategyType::SolendWithLM => Box::new(SolendWithLMHandler {}),
        _ => panic!(),
    }
}

#[async_trait]
pub trait StrategyHandler {
    async fn withdraw_directly_from_strategy(
        &self,
        url: Cluster,
        // payer: &Keypair,
        strategy: Pubkey,
        token_mint: Pubkey,
        base: Pubkey,
        partner: String,
        amount: u64,
    ) -> Result<()>;
}
