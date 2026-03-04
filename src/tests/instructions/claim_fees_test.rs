use super::super::test_helpers::create_ata;
use crate::tests::instructions::create_market_vault_test::MarketVaultResult;
use crate::tests::instructions::settle_market_test::SettleMarketResult;
use crate::tests::instructions::stake_outcome_tokens_test::StakeOutcomeTokensResult;
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_claim_fees(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
    stake: &StakeOutcomeTokensResult,
    settle: &SettleMarketResult,
) {
    /*
        briber,
        market,
        outcome_yes_mint,
        outcome_no_mint,
        briber_outcome_yes,
        briber_outcome_no,
        outcome_yes_vault,
        outcome_no_vault,
        _token_program,
        _system_program,
    */

    // Create briber ATAs for outcome tokens
    let (briber_outcome_yes_ata, briber_outcome_yes_ata_account) =
        create_ata(market.outcome_yes_mint_address, market.briber, 0);
    let (briber_outcome_no_ata, briber_outcome_no_ata_account) =
        create_ata(market.outcome_no_mint_address, market.briber, 0);

    let ix_accounts = vec![
        AccountMeta::new(market.briber, true),
        AccountMeta::new(settle.market_address, true),
        AccountMeta::new_readonly(market.outcome_yes_mint_address, false),
        AccountMeta::new_readonly(market.outcome_no_mint_address, false),
        AccountMeta::new(briber_outcome_yes_ata, false),
        AccountMeta::new(briber_outcome_no_ata, false),
        AccountMeta::new(market.outcome_yes_vault_address, false),
        AccountMeta::new(market.outcome_no_vault_address, false),
        AccountMeta::new_readonly(init.token_program, false),
        AccountMeta::new_readonly(init.system_program, false),
    ];

    let ix_data = vec![9u8];

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            market.briber,
            market.result.get_account(&market.briber).unwrap().clone(),
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
            market.outcome_yes_mint_address,
            market
                .result
                .get_account(&market.outcome_yes_mint_address)
                .unwrap()
                .clone(),
        ),
        (
            market.outcome_no_mint_address,
            market
                .result
                .get_account(&market.outcome_no_mint_address)
                .unwrap()
                .clone(),
        ),
        (briber_outcome_yes_ata, briber_outcome_yes_ata_account),
        (briber_outcome_no_ata, briber_outcome_no_ata_account),
        (
            market.outcome_yes_vault_address,
            stake
                .result
                .get_account(&market.outcome_yes_vault_address)
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
