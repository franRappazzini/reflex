use crate::tests::instructions::stake_outcome_tokens_test::StakeOutcomeTokensResult;
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub struct UnstakeOutcomeTokensResult {
    pub result: mollusk_svm::result::InstructionResult,
    pub farmer: solana_address::Address,
    pub market_address: solana_address::Address,
    pub farmer_position: solana_address::Address,
    pub yes_mint_address: solana_address::Address,
    pub farmer_yes_ata: solana_address::Address,
    pub market_yes_vault: solana_address::Address,
}

pub fn run_unstake_outcome_tokens(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    stake: &StakeOutcomeTokensResult,
) -> UnstakeOutcomeTokensResult {
    /*
        farmer,
        market,
        farmer_position,
        outcome_mint,
        farmer_outcome_ata,
        market_outcome_vault,
        _token_program
    */

    let ix_accounts = vec![
        AccountMeta::new(stake.farmer, true),
        AccountMeta::new(stake.market_address, true),
        AccountMeta::new(stake.farmer_position, false),
        AccountMeta::new_readonly(stake.yes_mint_address, false),
        AccountMeta::new(stake.farmer_yes_ata, false),
        AccountMeta::new(stake.market_yes_vault, false),
        AccountMeta::new_readonly(init.token_program, false),
    ];

    let mut ix_data = vec![3u8];
    let outcome_yes_amount = 500_000_000u64;
    ix_data.extend_from_slice(&outcome_yes_amount.to_le_bytes());

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            stake.farmer,
            stake.result.get_account(&stake.farmer).unwrap().clone(),
        ),
        (
            stake.market_address,
            stake
                .result
                .get_account(&stake.market_address)
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
            stake.yes_mint_address,
            stake
                .result
                .get_account(&stake.yes_mint_address)
                .unwrap()
                .clone(),
        ),
        (
            stake.farmer_yes_ata,
            stake
                .result
                .get_account(&stake.farmer_yes_ata)
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

    UnstakeOutcomeTokensResult {
        result,
        farmer: stake.farmer,
        market_address: stake.market_address,
        farmer_position: stake.farmer_position,
        yes_mint_address: stake.yes_mint_address,
        farmer_yes_ata: stake.farmer_yes_ata,
        market_yes_vault: stake.market_yes_vault,
    }
}
