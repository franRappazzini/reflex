use super::super::test_helpers::create_ata;
use crate::tests::instructions::create_market_vault_test::MarketVaultResult;
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_withdraw_treasury(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
) {
    /*
        authority,
        config,
        mint,
        treasury,
        authority_ata,
        _token_program,
    */

    let (authority_wsol_ata, authority_wsol_ata_account) =
        create_ata(init.wsol_mint_address, init.authority, 0);

    let ix_accounts = vec![
        AccountMeta::new(init.authority, true),
        AccountMeta::new(init.config, true),
        AccountMeta::new_readonly(init.wsol_mint_address, false),
        AccountMeta::new(init.wsol_treasury, false),
        AccountMeta::new(authority_wsol_ata, false),
        AccountMeta::new_readonly(init.token_program, false),
    ];

    let ix_data = vec![10u8];

    let ix = Instruction::new_with_bytes(init.program_id, &ix_data, ix_accounts);

    let accounts = [
        (
            init.authority,
            init.result.get_account(&init.authority).unwrap().clone(),
        ),
        (
            init.config,
            market.result.get_account(&init.config).unwrap().clone(),
        ),
        (
            init.wsol_mint_address,
            init.result
                .get_account(&init.wsol_mint_address)
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
        (authority_wsol_ata, authority_wsol_ata_account),
        (
            init.token_program,
            init.result
                .get_account(&init.token_program)
                .unwrap()
                .clone(),
        ),
    ];

    init.mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
}
