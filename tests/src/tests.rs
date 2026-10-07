use anchor_client::{
    anchor_lang::prelude::{program_pack::Pack, system_instruction::create_account},
    Client, Cluster, CommitmentConfig,
};
use solana_keypair::{read_keypair_file, Keypair};
use solana_pubkey::{pubkey, Pubkey};
use solana_signer::Signer;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::instruction::{initialize_mint2, mint_to};

#[test]
fn test_initialize() {
    let program_id = "8i3X3C7WaRGxKLTpoecUqmPEueMULXVTRzEJFuyGrTHd";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let authority = read_keypair_file(&anchor_wallet).unwrap();
    let creator = Keypair::new();

    let client =
        Client::new_with_options(Cluster::Localnet, &authority, CommitmentConfig::confirmed());

    let program_id = Pubkey::try_from(program_id).unwrap();
    let program = client.program(program_id).unwrap();

    program
        .rpc()
        .request_airdrop(&creator.pubkey(), 1_000_000_000)
        .unwrap(); // 1 sol

    let config = Pubkey::find_program_address(&[reflex::constants::CONFIG_SEED], &program_id).0;

    let initialize_tx = program
        .request()
        .accounts(reflex::accounts::Initialize {
            authority: authority.pubkey(),
            config,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::Initialize {
            fee_bps: 500,
            briber_fee_bps: 500,
        })
        .send()
        .expect("");

    println!("Initialize tx signature {}", initialize_tx);

    let predict_program = pubkey!("prediCtPZCttYMvm2W3PtxmMxLmT1dtN7riU6Cxh6tM");
    let market = pubkey!("JE36UZNtBHAQzTHG2o8xp2hx9K5VxoQNu2qkVXfMJB4q"); // harcoded
    let pool = Pubkey::find_program_address(
        &[reflex::constants::POOL_SEED, market.as_ref()],
        &program_id,
    )
    .0;
    let pool_vault =
        Pubkey::find_program_address(&[reflex::constants::VAULT_SEED, pool.as_ref()], &program_id)
            .0;

    let token_program = spl_token_interface::ID;

    let mint = Keypair::new();

    let lamports = program
        .rpc()
        .get_minimum_balance_for_rent_exemption(spl_token_interface::state::Mint::LEN)
        .unwrap();

    let create_mint_account = create_account(
        &creator.pubkey(),
        &mint.pubkey(),
        lamports,
        spl_token_interface::state::Mint::LEN as u64,
        &token_program,
    );

    let initialize_mint_ix =
        initialize_mint2(&token_program, &mint.pubkey(), &creator.pubkey(), None, 6).unwrap();

    let signer_ata = get_associated_token_address(&creator.pubkey(), &mint.pubkey());
    let create_ata_ix = create_associated_token_account(
        &creator.pubkey(),
        &creator.pubkey(),
        &mint.pubkey(),
        &token_program,
    );

    let mint_to_ix = mint_to(
        &token_program,
        &mint.pubkey(),
        &signer_ata,
        &creator.pubkey(),
        &[&creator.pubkey()],
        10_000_000_000, // 10k usdc
    )
    .unwrap();

    let create_pool_tx = program
        .request()
        .accounts(reflex::accounts::CreatePool {
            signer: creator.pubkey(),
            config,
            pool,
            market,
            mint: mint.pubkey(),
            pool_vault,
            signer_ata,
            predict_program,
            token_program,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::CreatePool {
            amount: 1_000_000_000, // 1k usdc
        })
        .instruction(create_mint_account)
        .instruction(initialize_mint_ix)
        .instruction(create_ata_ix)
        .instruction(mint_to_ix)
        .payer(&creator)
        .signer(mint)
        .send()
        .expect("");

    println!("Create Pool tx signature {}", create_pool_tx);
}
