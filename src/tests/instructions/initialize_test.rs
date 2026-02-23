use super::super::test_helpers::{base_data, create_mint, TestBase};
use crate::constants;
use mollusk_svm::result::Check;
use solana_account::Account;
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};

pub struct InitializeResult {
    pub result: mollusk_svm::result::InstructionResult,
    pub program_id: Address,
    pub config: Address,
    pub wsol_treasury: Address,
    pub wsol_mint_address: Address,
    pub usdc_address: Address,
    pub wsol_account: Account,
    pub usdc_account: Account,
    pub token_program: Address,
    pub system_program: Address,
    pub authority: Address,
    pub authority_account: Account,
    pub mollusk: mollusk_svm::Mollusk,
}

pub fn run_initialize() -> InitializeResult {
    let TestBase {
        program_id,
        mollusk,
        system_program,
        system_account,
        authority,
        authority_account,
        token_program,
        token_program_account,
        ..
    } = base_data();

    let (config, _) = Address::find_program_address(&[constants::CONFIG_SEED], &program_id);
    let config_account = Account::default();

    let fee_bps = 500u16;
    let briber_fee_bps = 500u16;
    let mut ix_data = vec![0u8];
    ix_data.extend_from_slice(&[fee_bps.to_le_bytes(), briber_fee_bps.to_le_bytes()].concat());

    let wsol_mint_address = Address::new_unique();
    let wsol_account = create_mint(wsol_mint_address, 10_000_000_000_000, 9);

    let usdc_address = Address::new_unique();
    let usdc_account = create_mint(usdc_address, 10_000_000_000_000, 6);

    let (wsol_treasury_address, _) = Address::find_program_address(
        &[constants::TREASURY_SEED, wsol_mint_address.as_ref()],
        &program_id,
    );
    let wsol_treasury_account = Account::default();

    let (usdc_treasury_address, _) = Address::find_program_address(
        &[constants::TREASURY_SEED, usdc_address.as_ref()],
        &program_id,
    );
    let usdc_treasury_account = Account::default();

    let ix_accounts = vec![
        AccountMeta::new(authority, true),
        AccountMeta::new(config, false),
        AccountMeta::new_readonly(wsol_mint_address, false),
        AccountMeta::new_readonly(usdc_address, false),
        AccountMeta::new(wsol_treasury_address, false),
        AccountMeta::new(usdc_treasury_address, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let ix = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

    let accounts = [
        (authority, authority_account.clone()),
        (config, config_account),
        (wsol_mint_address, wsol_account.clone()),
        (usdc_address, usdc_account.clone()),
        (wsol_treasury_address, wsol_treasury_account.clone()),
        (usdc_treasury_address, usdc_treasury_account.clone()),
        (token_program, token_program_account.clone()),
        (system_program, system_account.clone()),
    ];

    let result = mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()]);

    InitializeResult {
        program_id,
        result,
        config,
        wsol_treasury: wsol_treasury_address,
        wsol_mint_address,
        usdc_address,
        wsol_account,
        usdc_account,
        token_program,
        system_program,
        authority,
        authority_account,
        mollusk,
    }
}
