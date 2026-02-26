use crate::tests::instructions::create_market_vault_test::MarketVaultResult;
use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_settle_market(
    init: &mut crate::tests::instructions::initialize_test::InitializeResult,
    market: &MarketVaultResult,
) {
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

    init.mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
}
