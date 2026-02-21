//! Helpers reutilizables para tests SBF de Reflex

use mollusk_svm::{program, Mollusk};
use mollusk_svm_programs_token::{
    associated_token::create_account_for_associated_token_account, token::create_account_for_mint,
};
use solana_account::Account;
use solana_address::{address, Address};

/// Estructura con los datos base para tests
pub struct TestBase {
    pub program_id: Address,
    pub mollusk: Mollusk,
    pub system_program: Address,
    pub system_account: Account,
    pub authority: Address,
    pub authority_account: Account,
    pub token_program: Address,
    pub token_program_account: Account,
    pub token2022_program: Address,
    pub token2022_program_account: Account,
    pub associated_token_program: Address,
    pub associated_token_program_account: Account,
}

/// Inicializa el entorno base de cuentas y programas para los tests
pub fn base_data() -> TestBase {
    let program_id = address!("7zogcJaEsucGbcnZz4o4ARRbeF8AUU1RUP7zsAJ68wK7");
    let mut mollusk = Mollusk::new(&program_id, "target/deploy/reflex");

    // Add the SPL Token Program
    mollusk_svm_programs_token::token::add_program(&mut mollusk);
    // Add the Token2022 Program
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);
    // Add the Associated Token Program
    mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);

    let (system_program, system_account) = program::keyed_account_for_system_program();
    let (token_program, token_program_account) = mollusk_svm_programs_token::token::keyed_account();
    let (token2022_program, token2022_program_account) =
        mollusk_svm_programs_token::token2022::keyed_account();
    let (associated_token_program, associated_token_program_account) =
        mollusk_svm_programs_token::associated_token::keyed_account();

    let authority = Address::new_unique();
    let authority_account = Account::new(100_000_000_000, 0, &system_program);

    TestBase {
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
    }
}

/// Helper para crear un mint SPL Token
pub fn create_mint(mint_authority: Address, supply: u64, decimals: u8) -> Account {
    let mint_data = spl_token::state::Mint {
        mint_authority: solana_program::program_option::COption::Some(mint_authority),
        supply,
        decimals,
        is_initialized: true,
        freeze_authority: solana_program::program_option::COption::None,
    };
    create_account_for_mint(mint_data)
}

/// Helper para crear una cuenta asociada de token (ATA)
pub fn create_ata(mint: Address, owner: Address, amount: u64) -> (Address, Account) {
    create_account_for_associated_token_account(spl_token::state::Account {
        mint,
        owner,
        amount,
        delegate: solana_program::program_option::COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: solana_program::program_option::COption::None,
        delegated_amount: 0,
        close_authority: solana_program::program_option::COption::None,
    })
}
