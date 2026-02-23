use crate::tests::instructions::{
    create_market_vault_test::MarketVaultResult,
    unstake_outcome_tokens_test::UnstakeOutcomeTokensResult,
};
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_add_incentives(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
) {
    /*
        briber,
        market,
        incentive_mint,
        briber_ata,
        market_incentive_vault,
        incentive_treasury,
        token_program
    */

    let ix_accounts = vec![
        AccountMeta::new(market.briber, true),
        AccountMeta::new(market.market_vault_address, true),
        AccountMeta::new_readonly(init.wsol_mint_address, false),
        AccountMeta::new(market.briber_ata_address, false),
        AccountMeta::new(market.market_incentive_vault_address, false),
        AccountMeta::new(init.wsol_treasury, false),
        AccountMeta::new_readonly(init.token_program, false),
    ];

    let mut ix_data = vec![4u8];
    let outcome_yes_amount = 500_000_000_000u64;
    ix_data.extend_from_slice(&outcome_yes_amount.to_le_bytes());

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            market.briber,
            market.result.get_account(&market.briber).unwrap().clone(),
        ),
        (
            market.market_vault_address,
            market
                .result
                .get_account(&market.market_vault_address)
                .unwrap()
                .clone(),
        ),
        (
            init.wsol_mint_address,
            init.result
                .get_account(&init.wsol_mint_address)
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
            init.wsol_treasury,
            market
                .result
                .get_account(&init.wsol_treasury)
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
    ];

    let result = init
        .mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
}
