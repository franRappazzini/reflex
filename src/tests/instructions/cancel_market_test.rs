use super::initialize_test::InitializeResult;
use crate::tests::instructions::{
    create_market_vault_test::MarketVaultResult,
    stake_outcome_tokens_test::StakeOutcomeTokensResult,
};
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_cancel_market(
    init: &mut InitializeResult,
    market: &MarketVaultResult,
    stake: Option<&StakeOutcomeTokensResult>,
    checks: &[Check],
) {
    /*
        briber
        market
        briber_ata
        market_incentive_vault
        token_program
        system_program
    */

    let ix_accounts = vec![
        AccountMeta::new(market.briber, true),
        AccountMeta::new(market.market_vault_address, true),
        AccountMeta::new(market.briber_ata_address, false),
        AccountMeta::new(market.market_incentive_vault_address, false),
        AccountMeta::new_readonly(init.token_program, false),
        AccountMeta::new_readonly(init.system_program, false),
    ];

    let ix_data = vec![5u8];

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let use_result = if stake.is_some() {
        stake.unwrap().result.clone()
    } else {
        market.result.clone()
    };

    let accounts = [
        (
            market.briber,
            market.result.get_account(&market.briber).unwrap().clone(),
        ),
        (
            market.market_vault_address,
            use_result
                .get_account(&market.market_vault_address)
                .unwrap()
                .clone(),
        ),
        (
            market.briber_ata_address,
            market
                .result
                .get_account(&market.briber_ata_address)
                .unwrap()
                .clone(),
        ),
        (
            market.market_incentive_vault_address,
            market
                .result
                .get_account(&market.market_incentive_vault_address)
                .unwrap()
                .clone(),
        ),
        (
            init.token_program,
            init.result
                .get_account(&init.token_program)
                .unwrap()
                .clone(),
        ),
        (
            init.system_program,
            init.result
                .get_account(&init.system_program)
                .unwrap()
                .clone(),
        ),
    ];

    init.mollusk
        .process_and_validate_instruction(&ix, &accounts, checks);
}
