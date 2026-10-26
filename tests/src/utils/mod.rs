use std::{path::PathBuf, rc::Rc};

use anchor_client::{
    anchor_lang::prelude::{program_pack::Pack, system_instruction::create_account},
    Client, Cluster, CommitmentConfig, Instruction, Program,
};

use reflex::{WORLD_MARKET_SIZE, WORLD_NO_MINT_OFFSET, WORLD_PROGRAM_ID, WORLD_YES_MINT_OFFSET};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::{pubkey, Pubkey};
use solana_signer::Signer;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account,
};
use spl_token_interface::instruction::{initialize_mint2, mint_to};

// -----------------------------------------------------------------------------
// constants
// -----------------------------------------------------------------------------
pub const PROGRAM_ID: Pubkey = pubkey!("HQqk2aUfK95RFvwwQn3C1cP4cev18U9mWMW4nC35xJj1");
pub const TOKEN_2022_ID: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub const ONE_SOL: u64 = 1_000_000_000;
const SURFPOOL_RPC_URL: &str = "http://127.0.0.1:8899";
pub const USDC_DECIMALS: u8 = 6;
pub const INITIAL_MINT_AMOUNT: u64 = 10_000_000_000; // 10k USDC
pub const INITIAL_OUTCOME_AMOUNT: u64 = 5_000_000_000;
pub const INITIAL_STAKE_AMOUNT: u64 = 1_000_000_000;
pub const INITIAL_POOL_FUNDING_AMOUNT: u64 = 1_000_000_000;
pub const ADDITIONAL_INCENTIVE_AMOUNT: u64 = 500_000_000;

// -----------------------------------------------------------------------------
// structs
// -----------------------------------------------------------------------------
pub struct TestAccounts {
    pub authority: Keypair,
    pub creator: Keypair,
    pub staker: Keypair,
}

pub struct TestPdas {
    pub config: Pubkey,
    pub pool: Pubkey,
    pub pool_vault: Pubkey,
    pub position: Pubkey,
    pub funder: Pubkey,
}

pub struct TestContext {
    pub client: Client<Rc<Keypair>>,
    pub program: Program<Rc<Keypair>>,
    pub accounts: TestAccounts,
    pub pdas: TestPdas,
    pub world_random_market: Pubkey,
}

pub struct MintSetup {
    pub mint: Keypair,
    pub ata: Pubkey,
    pub create_account_ix: Instruction,
    pub init_mint_ix: Instruction,
    pub create_ata_ix: Instruction,
    pub mint_to_ix: Instruction,
}

pub struct OutcomeSetup {
    pub mint: Pubkey,
    pub ata: Pubkey,
    pub vault: Pubkey,
}

pub struct IncentiveSetup {
    pub mint: Pubkey,
    pub ata: Pubkey,
}

pub struct PoolSetup {
    pub incentive: IncentiveSetup,
    pub yes: OutcomeSetup,
    pub no: OutcomeSetup,
}

pub fn fetch_account<T: anchor_client::anchor_lang::AccountDeserialize>(
    ctx: &TestContext,
    address: &Pubkey,
) -> T {
    let account = ctx
        .program
        .rpc()
        .get_account(address)
        .expect("failed to fetch account");
    let mut data: &[u8] = &account.data;

    T::try_deserialize(&mut data).expect("failed to deserialize account")
}

pub fn fetch_market_outcome_mints(ctx: &TestContext) -> (Pubkey, Pubkey) {
    let market = ctx
        .program
        .rpc()
        .get_account(&ctx.world_random_market)
        .expect("failed to fetch World market account");
    assert_eq!(
        market.owner, WORLD_PROGRAM_ID,
        "unexpected World market owner"
    );
    assert_eq!(
        market.data.len(),
        WORLD_MARKET_SIZE,
        "unexpected World market account size"
    );

    let mint_at = |offset: usize| {
        let bytes: [u8; 32] = market.data[offset..offset + 32]
            .try_into()
            .expect("World market mint field must be 32 bytes");
        Pubkey::new_from_array(bytes)
    };

    (
        mint_at(WORLD_YES_MINT_OFFSET),
        mint_at(WORLD_NO_MINT_OFFSET),
    )
}

pub fn set_token_account_balance(
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    token_program: &Pubkey,
) {
    let response: serde_json::Value = reqwest::blocking::Client::new()
        .post(SURFPOOL_RPC_URL)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setTokenAccount",
            "params": [
                owner.to_string(),
                mint.to_string(),
                { "amount": amount, "state": "initialized" },
                token_program.to_string()
            ]
        }))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .unwrap_or_else(|error| panic!("failed to set token account balance: {error}"))
        .json()
        .unwrap_or_else(|error| panic!("invalid Surfpool token account response: {error}"));

    if let Some(error) = response.get("error") {
        panic!("failed to set token account balance: {error}");
    }
}

pub fn token_amount(owner: &Pubkey, data: &[u8]) -> u64 {
    if owner == &TOKEN_2022_ID {
        spl_token_2022_interface::extension::StateWithExtensions::<
            spl_token_2022_interface::state::Account,
        >::unpack(data)
        .expect("failed to decode Token-2022 account")
        .base
        .amount
    } else {
        spl_token_interface::state::Account::unpack(data)
            .expect("failed to decode SPL Token account")
            .amount
    }
}

pub fn derive_outcome_vault(program_id: &Pubkey, pool: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[reflex::constants::VAULT_SEED, pool.as_ref(), mint.as_ref()],
        program_id,
    )
    .0
}

pub fn derive_funder_pda(program_id: &Pubkey, pool: &Pubkey, funder: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            reflex::constants::FUNDER_SEED,
            pool.as_ref(),
            funder.as_ref(),
        ],
        program_id,
    )
    .0
}

// -----------------------------------------------------------------------------
// utils fns
// -----------------------------------------------------------------------------

fn get_keypair_path(filename: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    PathBuf::from(manifest_dir).join("keypairs").join(filename)
}

pub fn load_accounts() -> TestAccounts {
    let anchor_wallet = std::env::var("ANCHOR_WALLET")
        .expect("ANCHOR_WALLET must be set to the authority keypair path");

    let authority = read_keypair_file(&anchor_wallet).unwrap_or_else(|error| {
        panic!("failed to read authority keypair {anchor_wallet}: {error}")
    });

    let creator_path = get_keypair_path("creator.json");
    let staker_path = get_keypair_path("staker.json");

    let creator = read_keypair_file(&creator_path)
        .unwrap_or_else(|error| panic!("failed to read creator keypair: {error}"));
    let staker = read_keypair_file(&staker_path)
        .unwrap_or_else(|error| panic!("failed to read staker keypair: {error}"));

    TestAccounts {
        authority,
        creator,
        staker,
    }
}

pub fn derive_pdas(
    program_id: &Pubkey,
    market: &Pubkey,
    staker_address: &Pubkey,
    funder_address: &Pubkey,
) -> TestPdas {
    let (config, _) = Pubkey::find_program_address(&[reflex::constants::CONFIG_SEED], program_id);
    let (pool, _) =
        Pubkey::find_program_address(&[reflex::constants::POOL_SEED, market.as_ref()], program_id);
    let (pool_vault, _) =
        Pubkey::find_program_address(&[reflex::constants::VAULT_SEED, pool.as_ref()], program_id);
    let (position, _) = Pubkey::find_program_address(
        &[
            reflex::constants::POSITION_SEED,
            pool.as_ref(),
            staker_address.as_ref(),
        ],
        program_id,
    );
    let (funder, _) = Pubkey::find_program_address(
        &[
            reflex::constants::FUNDER_SEED,
            pool.as_ref(),
            funder_address.as_ref(),
        ],
        program_id,
    );

    TestPdas {
        config,
        pool,
        pool_vault,
        position,
        funder,
    }
}

// -----------------------------------------------------------------------------
// initial setup fn
// -----------------------------------------------------------------------------
pub fn setup() -> TestContext {
    let accounts = load_accounts();

    let authority_rc = Rc::new(accounts.authority.insecure_clone());
    let client = Client::new_with_options(
        Cluster::Localnet,
        authority_rc,
        CommitmentConfig::confirmed(),
    );

    let program = client
        .program(PROGRAM_ID)
        .expect("Error al instanciar el cliente del programa");

    fund_test_wallet(&accounts.creator.pubkey(), "creator");
    fund_test_wallet(&accounts.staker.pubkey(), "staker");

    let world_accounts = program
        .rpc()
        .get_program_accounts(&WORLD_PROGRAM_ID)
        .expect("Fail to get World program accounts");
    let (last_world_market_address, _) = world_accounts
        .iter()
        .filter(|(_, a)| a.data.len() == WORLD_MARKET_SIZE)
        .nth(16)
        .expect("Fail to get one account");

    let pdas = derive_pdas(
        &PROGRAM_ID,
        last_world_market_address,
        &accounts.staker.pubkey(),
        &accounts.creator.pubkey(),
    );

    let staker_funder = derive_funder_pda(&PROGRAM_ID, &pdas.pool, &accounts.staker.pubkey());
    for address in [
        pdas.config,
        pdas.pool,
        pdas.pool_vault,
        pdas.position,
        pdas.funder,
        staker_funder,
    ] {
        reset_test_account(&address);
    }

    println!("world market address: {last_world_market_address}");

    TestContext {
        client,
        program,
        accounts,
        pdas,
        world_random_market: *last_world_market_address,
    }
}

fn fund_test_wallet(address: &Pubkey, name: &str) {
    let response: serde_json::Value = reqwest::blocking::Client::new()
        .post(SURFPOOL_RPC_URL)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_setAccount",
            "params": [address.to_string(), { "lamports": ONE_SOL }]
        }))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .unwrap_or_else(|error| panic!("failed to fund {name} through Surfpool RPC: {error}"))
        .json()
        .unwrap_or_else(|error| panic!("invalid Surfpool funding response for {name}: {error}"));

    if let Some(error) = response.get("error") {
        panic!("failed to fund {name} through Surfpool RPC: {error}");
    }
}

fn reset_test_account(address: &Pubkey) {
    let response: serde_json::Value = reqwest::blocking::Client::new()
        .post(SURFPOOL_RPC_URL)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "surfnet_resetAccount",
            "params": [address.to_string()]
        }))
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .unwrap_or_else(|error| panic!("failed to reset test account {address}: {error}"))
        .json()
        .unwrap_or_else(|error| panic!("invalid Surfpool reset response for {address}: {error}"));

    if let Some(error) = response.get("error") {
        panic!("failed to reset test account {address}: {error}");
    }
}

// -----------------------------------------------------------------------------
// token helpers
// -----------------------------------------------------------------------------
pub fn prepare_mint_and_ata(
    program: &Program<Rc<Keypair>>,
    owner: &Pubkey,
    decimals: u8,
    amount: u64,
    token_program: Pubkey,
) -> MintSetup {
    let mint = Keypair::new();

    let lamports = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(spl_token_interface::state::Mint::LEN)
        .expect("Error obteniendo rent exemption");

    let create_account_ix = create_account(
        owner,
        &mint.pubkey(),
        lamports,
        spl_token_interface::state::Mint::LEN as u64,
        &token_program,
    );

    let init_mint_ix = if token_program == TOKEN_2022_ID {
        spl_token_2022_interface::instruction::initialize_mint2(
            &token_program,
            &mint.pubkey(),
            owner,
            None,
            decimals,
        )
    } else {
        initialize_mint2(&token_program, &mint.pubkey(), owner, None, decimals)
    }
    .expect("Error creando instrucción initialize_mint2");

    let ata = get_associated_token_address_with_program_id(owner, &mint.pubkey(), &token_program);
    let create_ata_ix =
        create_associated_token_account(owner, owner, &mint.pubkey(), &token_program);

    let mint_to_ix = if token_program == TOKEN_2022_ID {
        spl_token_2022_interface::instruction::mint_to(
            &token_program,
            &mint.pubkey(),
            &ata,
            owner,
            &[owner],
            amount,
        )
    } else {
        mint_to(
            &token_program,
            &mint.pubkey(),
            &ata,
            owner,
            &[owner],
            amount,
        )
    }
    .expect("Error creando instrucción mint_to");

    MintSetup {
        mint,
        ata,
        create_account_ix,
        init_mint_ix,
        create_ata_ix,
        mint_to_ix,
    }
}
