mod tests {
    use mollusk_svm::{program, result::Check, Mollusk};
    use mollusk_svm_programs_token::token::create_account_for_mint;
    use solana_account::Account;
    use solana_address::{address, Address};
    use solana_instruction::{AccountMeta, Instruction};

    use crate::constants;

    fn base_data() -> (
        Address,
        Mollusk,
        Address,
        Account,
        Address,
        Account,
        Address,
        Account,
        Address,
        Account,
        Address,
        Account,
    ) {
        let program_id = address!("7zogcJaEsucGbcnZz4o4ARRbeF8AUU1RUP7zsAJ68wK7");
        let mut mollusk = Mollusk::new(&program_id, "target/deploy/reflex");

        // Add the SPL Token Program
        mollusk_svm_programs_token::token::add_program(&mut mollusk);
        // Add the Token2022 Program
        mollusk_svm_programs_token::token2022::add_program(&mut mollusk);
        // Add the Associated Token Program
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

        let (system_program, system_account) = program::keyed_account_for_system_program();

        // SPL Token Program
        let (token_program, token_program_account) =
            mollusk_svm_programs_token::token::keyed_account();
        // Token2022 Program
        let (token2022_program, token2022_program_account) =
            mollusk_svm_programs_token::token2022::keyed_account();
        // Associated Token Program
        let (associated_token_program, associated_token_program_account) =
            mollusk_svm_programs_token::associated_token::keyed_account();

        let authority = Address::new_unique();
        let authority_account = Account::new(100_000_000_000, 0, &system_program);

        return (
            program_id,
            mollusk,
            system_program,
            system_account,
            authority,
            authority_account,
            token_program,
            token_program_account,
            token2022_program,
            token2022_program_account,
            associated_token_program,
            associated_token_program_account,
        );
    }

    #[test]
    fn initialize() {
        let (
            program_id,
            mollusk,
            system_program,
            system_account,
            authority,
            authority_account,
            token_program,
            token_program_account,
            token2022_program,
            token2022_program_account,
            associated_token_program,
            associated_token_program_account,
        ) = base_data();

        let (config, _) = Address::find_program_address(&[constants::CONFIG_SEED], &program_id);
        let config_account = Account::default();

        // 0 - fee_bps: u16 - briber_fee_bps: u16
        let fee_bps = 500u16;
        let briber_fee_bps = 500u16;
        let mut ix_data = vec![0u8];
        ix_data.extend_from_slice(&[fee_bps.to_le_bytes(), briber_fee_bps.to_le_bytes()].concat());

        /*
            authority: &'a AccountView,
            config: &'a AccountView,
            wsol_mint: &'a AccountView,
            usdc_mint: &'a AccountView,
            wsol_treasury: &'a AccountView,
            usdc_treasury: &'a AccountView,
            token_program: &'a AccountView,
            system_program: &'a AccountView,
        */

        let wsol_address = Address::new_unique();

        let wsol_data = spl_token::state::Mint {
            mint_authority: solana_program::program_option::COption::Some(wsol_address),
            supply: 10_000_000_000_000,
            decimals: 9,
            is_initialized: true,
            freeze_authority: solana_program::program_option::COption::None,
        };
        let wsol_account = create_account_for_mint(wsol_data);

        let usdc_address = Address::new_unique();

        let usdc_data = spl_token::state::Mint {
            mint_authority: solana_program::program_option::COption::Some(usdc_address),
            supply: 10_000_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: solana_program::program_option::COption::None,
        };
        let usdc_account = create_account_for_mint(usdc_data);

        let (wsol_treasury_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, wsol_address.as_ref()],
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
            AccountMeta::new_readonly(wsol_address, false),
            AccountMeta::new_readonly(usdc_address, false),
            AccountMeta::new(wsol_treasury_address, false),
            AccountMeta::new(usdc_treasury_address, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        let ix = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

        let accounts = [
            (authority, authority_account),
            (config, config_account),
            (wsol_address, wsol_account),
            (usdc_address, usdc_account),
            (wsol_treasury_address, wsol_treasury_account),
            (usdc_treasury_address, usdc_treasury_account),
            (token_program, token_program_account),
            (system_program, system_account),
        ];

        mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
    }

    #[test]
    fn create_vault_market() {}
}
