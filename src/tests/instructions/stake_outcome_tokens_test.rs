use super::super::test_helpers::create_ata;
use super::create_market_vault_test::MarketVaultResult;
use crate::constants;
use mollusk_svm::result::Check;
use solana_account::Account;
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};

pub struct StakeOutcomeTokensResult {
    pub result: mollusk_svm::result::InstructionResult,
    pub farmer: Address,
    pub farmer_position: Address,
    pub farmer_yes_ata: Address,
    pub market_address: Address,
    pub yes_mint_address: Address,
    pub no_mint_address: Address,
    pub market_yes_vault: Address,
    // pub market_no_vault: Address,
}

pub fn run_stake_outcome_tokens(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
) -> StakeOutcomeTokensResult {
    let farmer = Address::new_unique();
    let farmer_account = Account::new(100_000_000_000, 0, &init.system_program);

    let market_vault_account = market
        .result
        .get_account(&market.market_vault_address)
        .unwrap();

    let (farmer_position_address, _) = Address::find_program_address(
        &[
            constants::FARMER_POSITION_SEED,
            market.market_vault_address.as_ref(),
            farmer.as_ref(),
        ],
        &init.program_id,
    );
    let farmer_position_account = Account::default();

    let (farmer_yes_ata_address, farmer_yes_ata_account) =
        create_ata(market.outcome_yes_mint_address, farmer, 1_100_000_000);

    let outcome_yes_vault_address = market.outcome_yes_vault_address;
    let outcome_yes_vault_account = market
        .result
        .get_account(&outcome_yes_vault_address)
        .unwrap();

    let ix_accounts = vec![
        AccountMeta::new(farmer, true),
        AccountMeta::new(market.market_vault_address, false),
        AccountMeta::new(farmer_position_address, false),
        AccountMeta::new_readonly(market.outcome_yes_mint_address, false),
        AccountMeta::new(outcome_yes_vault_address, false), // market vault yes vault
        AccountMeta::new(farmer_yes_ata_address, false),
        AccountMeta::new_readonly(init.token_program, false),
        AccountMeta::new_readonly(init.system_program, false),
    ];

    let mut ix_data = vec![2u8];
    let outcome_yes_amount = 1_000_000_000u64;
    ix_data.extend_from_slice(&outcome_yes_amount.to_le_bytes());

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (farmer, farmer_account.clone()),
        (market.market_vault_address, market_vault_account.clone()),
        (farmer_position_address, farmer_position_account.clone()),
        (
            market.outcome_yes_mint_address,
            market.outcome_yes_mint_account.clone(),
        ),
        (outcome_yes_vault_address, outcome_yes_vault_account.clone()), // market vault yes vault
        (farmer_yes_ata_address, farmer_yes_ata_account.clone()),
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

    let result = init
        .mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);

    StakeOutcomeTokensResult {
        result,
        farmer,
        farmer_position: farmer_position_address,
        farmer_yes_ata: farmer_yes_ata_address,
        market_address: market.market_vault_address,
        yes_mint_address: market.outcome_yes_mint_address,
        no_mint_address: market.outcome_no_mint_address,
        market_yes_vault: market.outcome_yes_vault_address,
    }
}
