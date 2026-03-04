use crate::tests::instructions::create_market_vault_test::MarketVaultResult;
use crate::tests::instructions::stake_outcome_tokens_test::StakeOutcomeTokensResult;
use mollusk_svm::result::Check;
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};

pub struct SettleMarketResult {
    pub result: mollusk_svm::result::InstructionResult,
    pub market_address: Address,
}

pub fn run_settle_market(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
) -> SettleMarketResult {
    /*
        authority,
        config
        market,
    */

    let ix_accounts = vec![
        AccountMeta::new(init.authority, true),
        AccountMeta::new(init.config, true),
        AccountMeta::new(market.market_vault_address, true),
    ];

    let ix_data = vec![6u8, 1u8]; // [discriminator, resolution]

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            init.authority,
            init.result.get_account(&init.authority).unwrap().clone(),
        ),
        (
            init.config,
            init.result.get_account(&init.config).unwrap().clone(),
        ),
        (
            market.market_vault_address,
            market
                .result
                .get_account(&market.market_vault_address)
                .unwrap()
                .clone(),
        ),
    ];

    let result = init
        .mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);

    SettleMarketResult {
        result,
        market_address: market.market_vault_address,
    }
}

pub fn run_settle_market_with_stake(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    stake: &StakeOutcomeTokensResult,
) -> SettleMarketResult {
    let ix_accounts = vec![
        AccountMeta::new(init.authority, true),
        AccountMeta::new(init.config, true),
        AccountMeta::new(stake.market_address, true),
    ];

    let ix_data = vec![6u8, 1u8];
    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            init.authority,
            init.result.get_account(&init.authority).unwrap().clone(),
        ),
        (
            init.config,
            init.result.get_account(&init.config).unwrap().clone(),
        ),
        (
            stake.market_address,
            stake
                .result
                .get_account(&stake.market_address)
                .unwrap()
                .clone(),
        ),
    ];

    let result = init
        .mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);

    SettleMarketResult {
        result,
        market_address: stake.market_address,
    }
}
