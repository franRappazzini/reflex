use super::super::test_helpers::{create_ata, create_mint};
use super::initialize::InitializeResult;
use crate::constants;
use mollusk_svm::result::Check;
use solana_account::Account;
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};

pub struct MarketVaultResult {
    pub result: mollusk_svm::result::InstructionResult,
    pub market_vault_address: Address,
    pub outcome_yes_mint_address: Address,
    pub outcome_no_mint_address: Address,
    pub outcome_yes_mint_account: Account,
    pub outcome_no_mint_account: Account,
    pub outcome_yes_vault_address: Address,
    pub outcome_yes_vault_account: Account,
    pub briber: Address,
    pub briber_account: Account,
    pub briber_ata_address: Address,
    pub briber_ata_account: Account,
}

pub fn run_create_market_vault(init: &mut InitializeResult) -> MarketVaultResult {
    let briber = Address::new_unique();
    let briber_account = Account::new(10_000_000_000_000, 0, &init.system_program);

    let config_account = init.result.get_account(&init.config).unwrap();
    let wsol_treasury_account = init.result.get_account(&init.wsol_treasury).unwrap();

    let (market_vault_address, _) = Address::find_program_address(
        &[constants::MARKET_VAULT_SEED, &1u64.to_le_bytes()],
        &crate::ID,
    );
    let market_vault_account = Account::default();

    let outcome_yes_mint_address = Address::new_unique();
    let outcome_yes_mint_account = create_mint(outcome_yes_mint_address, 10_000_000_000_000, 6);

    let outcome_no_mint_address = Address::new_unique();
    let outcome_no_mint_account = create_mint(outcome_no_mint_address, 10_000_000_000_000, 6);

    let (briber_ata_address, briber_ata_account) =
        create_ata(init.wsol_mint_address, briber, 1_000_000_000_000);

    let (market_vault_treasury_address, _) = Address::find_program_address(
        &[constants::TREASURY_SEED, market_vault_address.as_ref()],
        &crate::ID,
    );
    let market_vault_treasury_account = Account::default();

    let (outcome_yes_vault_address, _) = Address::find_program_address(
        &[
            constants::TREASURY_SEED,
            market_vault_address.as_ref(),
            outcome_yes_mint_address.as_ref(),
        ],
        &crate::ID,
    );
    let outcome_yes_vault_account = Account::default();

    let (outcome_no_vault_address, _) = Address::find_program_address(
        &[
            constants::TREASURY_SEED,
            market_vault_address.as_ref(),
            outcome_no_mint_address.as_ref(),
        ],
        &crate::ID,
    );
    let outcome_no_vault_account = Account::default();

    let ix_accounts = vec![
        AccountMeta::new(briber, true),
        AccountMeta::new(init.config, false),
        AccountMeta::new_readonly(init.wsol_mint_address, false), // incentive_mint
        AccountMeta::new(init.wsol_treasury, false),              // protocol wsol treasury
        AccountMeta::new(market_vault_address, false),
        AccountMeta::new_readonly(outcome_yes_mint_address, false),
        AccountMeta::new_readonly(outcome_no_mint_address, false),
        AccountMeta::new(briber_ata_address, false),
        AccountMeta::new(market_vault_treasury_address, false),
        AccountMeta::new(outcome_yes_vault_address, false),
        AccountMeta::new(outcome_no_vault_address, false),
        AccountMeta::new_readonly(init.token_program, false),
        AccountMeta::new_readonly(init.system_program, false),
    ];

    let mut ix_data = vec![1u8];
    let incentive_amount = 10_000_000_000u64;
    ix_data.extend_from_slice(&incentive_amount.to_le_bytes());

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (briber, briber_account.clone()),
        (init.config, config_account.clone()),
        (init.wsol_mint_address, init.wsol_account.clone()),
        (init.wsol_treasury, wsol_treasury_account.clone()),
        (market_vault_address, market_vault_account.clone()),
        (outcome_yes_mint_address, outcome_yes_mint_account.clone()),
        (outcome_no_mint_address, outcome_no_mint_account.clone()),
        (briber_ata_address, briber_ata_account.clone()),
        (
            market_vault_treasury_address,
            market_vault_treasury_account.clone(),
        ),
        (outcome_yes_vault_address, outcome_yes_vault_account.clone()),
        (outcome_no_vault_address, outcome_no_vault_account.clone()),
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

    MarketVaultResult {
        result,
        market_vault_address,
        outcome_yes_mint_address,
        outcome_no_mint_address,
        outcome_yes_mint_account,
        outcome_no_mint_account,
        outcome_yes_vault_address,
        outcome_yes_vault_account,
        briber,
        briber_account,
        briber_ata_address,
        briber_ata_account,
    }
}
