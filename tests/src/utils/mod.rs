use std::{path::PathBuf, rc::Rc};

use anchor_client::{
    anchor_lang::prelude::program_pack::Pack, Client, Cluster, CommitmentConfig, Program,
};

use reflex::{
    constants::{USDC_MINT_ID, WORLD_PROGRAM_ID},
    WORLD_MARKET_SIZE, WORLD_NO_MINT_OFFSET, WORLD_YES_MINT_OFFSET,
};
use reqwest::blocking::Client as HttpClient;
use serde_json::{json, Value};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::{pubkey, Pubkey};
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

// -----------------------------------------------------------------------------
// constants
// -----------------------------------------------------------------------------
pub const PROGRAM_ID: Pubkey = pubkey!("8i3X3C7WaRGxKLTpoecUqmPEueMULXVTRzEJFuyGrTHd");

pub const ONE_SOL: u64 = 1_000_000_000;
pub const INITIAL_OUTCOME_AMOUNT: u64 = 5_000_000_000;
pub const INITIAL_STAKE_AMOUNT: u64 = 1_000_000_000;
pub const INITIAL_POOL_FUNDING_AMOUNT: u64 = 1_000_000_000;
pub const ADDITIONAL_INCENTIVE_AMOUNT: u64 = 500_000_000;
const SURFNET_RPC_URL: &str = "http://127.0.0.1:8899";

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
    pub world_outcome_mints: WorldOutcomeMints,
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

#[derive(Clone, Copy)]
pub struct WorldOutcomeMints {
    pub yes: Pubkey,
    pub no: Pubkey,
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

pub fn fetch_world_outcome_mints(
    program: &Program<Rc<Keypair>>,
    market: &Pubkey,
) -> Result<WorldOutcomeMints, String> {
    let account = program
        .rpc()
        .get_account(market)
        .map_err(|error| format!("failed to fetch World market account: {error}"))?;

    if account.owner != WORLD_PROGRAM_ID {
        return Err(format!(
            "World market account has unexpected owner: {}",
            account.owner
        ));
    }
    if account.data.len() != WORLD_MARKET_SIZE {
        return Err(format!(
            "World market account has unexpected data length: {}",
            account.data.len()
        ));
    }

    let read_mint = |offset: usize| -> Result<Pubkey, String> {
        let bytes: [u8; 32] = account
            .data
            .get(offset..offset + 32)
            .ok_or_else(|| format!("World market mint data is missing at offset {offset}"))?
            .try_into()
            .map_err(|_| format!("invalid World market mint data at offset {offset}"))?;
        Ok(Pubkey::new_from_array(bytes))
    };

    Ok(WorldOutcomeMints {
        yes: read_mint(WORLD_YES_MINT_OFFSET)?,
        no: read_mint(WORLD_NO_MINT_OFFSET)?,
    })
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

    program
        .rpc()
        .request_airdrop(&accounts.creator.pubkey(), ONE_SOL)
        .expect("Falló el airdrop a creator");
    program
        .rpc()
        .request_airdrop(&accounts.staker.pubkey(), ONE_SOL)
        .expect("Falló el airdrop a staker");

    let world_accounts = program
        .rpc()
        .get_program_accounts(&WORLD_PROGRAM_ID)
        .expect("Fail to get World program accounts");
    let (last_world_market_address, _) = world_accounts
        .iter()
        .filter(|(_, a)| a.data.len() == WORLD_MARKET_SIZE)
        .last()
        .expect("Fail to get one account");
    let market = *last_world_market_address;
    let world_outcome_mints = fetch_world_outcome_mints(&program, &market)
        .expect("failed to fetch outcome mints from World market");

    let pdas = derive_pdas(
        &PROGRAM_ID,
        &market,
        &accounts.staker.pubkey(),
        &accounts.creator.pubkey(),
    );

    set_token_account(
        &accounts.creator.pubkey(),
        &USDC_MINT_ID,
        &spl_token_interface::ID,
        INITIAL_POOL_FUNDING_AMOUNT,
    );
    set_token_account(
        &accounts.staker.pubkey(),
        &USDC_MINT_ID,
        &spl_token_interface::ID,
        ADDITIONAL_INCENTIVE_AMOUNT,
    );
    for mint in [world_outcome_mints.yes, world_outcome_mints.no] {
        set_token_account(
            &accounts.staker.pubkey(),
            &mint,
            &spl_token_2022_interface::ID,
            INITIAL_OUTCOME_AMOUNT,
        );
        set_token_account(
            &accounts.creator.pubkey(),
            &mint,
            &spl_token_2022_interface::ID,
            0,
        );
    }

    println!("world market address: {market}");

    TestContext {
        client,
        program,
        accounts,
        pdas,
        world_random_market: market,
        world_outcome_mints,
    }
}

pub fn associated_token_address(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, token_program)
}

pub fn set_token_account(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey, amount: u64) {
    surfnet_rpc(
        "surfnet_setTokenAccount",
        json!([
            owner.to_string(),
            mint.to_string(),
            {
                "amount": amount,
                "state": "initialized"
            },
            token_program.to_string()
        ]),
    )
    .unwrap_or_else(|error| panic!("failed to seed token account for {owner} / {mint}: {error}"));
}

pub fn surfnet_rpc(method: &str, params: Value) -> Result<Value, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });
    let response: Value = HttpClient::new()
        .post(SURFNET_RPC_URL)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .map_err(|error| format!("failed to send Surfpool {method} request: {error}"))?
        .error_for_status()
        .map_err(|error| format!("Surfpool {method} request returned an HTTP error: {error}"))?
        .json()
        .map_err(|error| format!("failed to decode Surfpool {method} response: {error}"))?;

    if let Some(error) = response.get("error") {
        return Err(format!("Surfpool {method} failed: {error}"));
    }

    Ok(response)
}

pub fn token_balance(ctx: &TestContext, address: &Pubkey) -> u64 {
    let account = ctx
        .program
        .rpc()
        .get_account(address)
        .expect("failed to fetch token account");

    if account.owner == spl_token_2022_interface::ID {
        use spl_token_2022_interface::{
            extension::StateWithExtensions, state::Account as Token2022Account,
        };
        StateWithExtensions::<Token2022Account>::unpack(&account.data)
            .expect("failed to decode Token-2022 account")
            .base
            .amount
    } else {
        spl_token_interface::state::Account::unpack(&account.data)
            .expect("failed to decode token account")
            .amount
    }
}
