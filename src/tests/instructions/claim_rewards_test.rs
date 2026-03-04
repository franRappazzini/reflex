use super::super::test_helpers::create_ata;
use crate::tests::instructions::create_market_vault_test::MarketVaultResult;
use crate::tests::instructions::settle_market_test::SettleMarketResult;
use crate::tests::instructions::stake_outcome_tokens_test::StakeOutcomeTokensResult;
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_claim_rewards(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
    stake: &StakeOutcomeTokensResult,
    settle: &SettleMarketResult,
) {
    /*
        farmer,
        market,
        farmer_position,
        incentive_mint,
        outcome_yes_mint,
        outcome_no_mint,
        farmer_incentive_ata,
        farmer_outcome_yes_ata,
        farmer_outcome_no_ata,
        market_vault,
        outcome_yes_vault,
        outcome_no_vault,
        _associated_token_program,
        token_program,
        _system_program
    */

    let (farmer_incentive_ata, farmer_incentive_ata_account) =
        create_ata(init.wsol_mint_address, stake.farmer, 0);
    let (farmer_outcome_no_ata, farmer_outcome_no_ata_account) =
        create_ata(market.outcome_no_mint_address, stake.farmer, 0);

    let (associated_token_program, associated_token_program_account) =
        mollusk_svm_programs_token::associated_token::keyed_account();

    let ix_accounts = vec![
        AccountMeta::new(stake.farmer, true),
        AccountMeta::new(settle.market_address, true),
        AccountMeta::new(stake.farmer_position, false),
        AccountMeta::new_readonly(init.wsol_mint_address, false),
        AccountMeta::new_readonly(stake.yes_mint_address, false),
        AccountMeta::new_readonly(stake.no_mint_address, false),
        AccountMeta::new(farmer_incentive_ata, false),
        AccountMeta::new(stake.farmer_yes_ata, false),
        AccountMeta::new(farmer_outcome_no_ata, false),
        AccountMeta::new(market.market_incentive_vault_address, false),
        AccountMeta::new(stake.market_yes_vault, false),
        AccountMeta::new(market.outcome_no_vault_address, false),
        AccountMeta::new_readonly(associated_token_program, false),
        AccountMeta::new_readonly(init.token_program, false),
        AccountMeta::new_readonly(init.system_program, false),
    ];

    let ix_data = vec![8u8];

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            stake.farmer,
            stake.result.get_account(&stake.farmer).unwrap().clone(),
        ),
        (
            settle.market_address,
            settle
                .result
                .get_account(&settle.market_address)
                .unwrap()
                .clone(),
        ),
        (
            stake.farmer_position,
            stake
                .result
                .get_account(&stake.farmer_position)
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
            stake.yes_mint_address,
            stake
                .result
                .get_account(&stake.yes_mint_address)
                .unwrap()
                .clone(),
        ),
        (
            stake.no_mint_address,
            market
                .result
                .get_account(&stake.no_mint_address)
                .unwrap()
                .clone(),
        ),
        (farmer_incentive_ata, farmer_incentive_ata_account),
        (
            stake.farmer_yes_ata,
            stake
                .result
                .get_account(&stake.farmer_yes_ata)
                .unwrap()
                .clone(),
        ),
        (farmer_outcome_no_ata, farmer_outcome_no_ata_account),
        (
            market.market_incentive_vault_address,
            market
                .result
                .get_account(&market.market_incentive_vault_address)
                .unwrap()
                .clone(),
        ),
        (
            stake.market_yes_vault,
            stake
                .result
                .get_account(&stake.market_yes_vault)
                .unwrap()
                .clone(),
        ),
        (
            market.outcome_no_vault_address,
            market
                .result
                .get_account(&market.outcome_no_vault_address)
                .unwrap()
                .clone(),
        ),
        (associated_token_program, associated_token_program_account),
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
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
}
