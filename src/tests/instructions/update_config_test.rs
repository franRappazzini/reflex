use mollusk_svm::result::Check;
use solana_instruction::{AccountMeta, Instruction};

pub fn run_update_config(init: &mut crate::tests::instructions::initialize_test::InitializeResult) {
    /*
        authority,
        config
    */

    let ix_accounts = vec![
        AccountMeta::new(init.authority, true),
        AccountMeta::new(init.config, false),
    ];

    let fee_bps = 250u16;
    let briber_fee_bps = 250u16;
    let mut ix_data = vec![7u8];
    ix_data.extend_from_slice(&[fee_bps.to_le_bytes(), briber_fee_bps.to_le_bytes()].concat());

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
    ];

    init.mollusk
        .process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
}
