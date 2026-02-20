mod tests {
    use mollusk_svm::{program, result::Check, Mollusk};
    use solana_account::Account;
    use solana_address::{address, Address};
    use solana_instruction::{AccountMeta, Instruction};

    use crate::constants;

    #[test]
    fn initialize() {
        let program_id = address!("7zogcJaEsucGbcnZz4o4ARRbeF8AUU1RUP7zsAJ68wK7");
        let mollusk = Mollusk::new(&program_id, "target/deploy/reflex");

        let (system_program, system_account) = program::keyed_account_for_system_program();

        let authority = Address::new_unique();
        let authority_account = Account::new(100_000_000_000, 0, &system_program);

        let (config, _) = Address::find_program_address(&[constants::CONFIG_SEED], &program_id);
        let config_account = Account::default();

        // 0 - fee_bps: u16 - briber_fee_bps: u16
        let fee_bps = 500u16;
        let briber_fee_bps = 500u16;
        let mut ix_data = vec![0u8];
        ix_data.extend_from_slice(&[fee_bps.to_le_bytes(), briber_fee_bps.to_le_bytes()].concat());

        // authority - config - system_program
        let ix_accounts = vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(config, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        let ix = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

        let accounts = [
            (authority, authority_account),
            (config, config_account),
            (system_program, system_account),
        ];

        mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
    }
}
