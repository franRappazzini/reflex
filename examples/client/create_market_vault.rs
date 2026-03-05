use solana_address::Address;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;

use super::helpers;
use super::initialize::InitializeResult;

pub struct CreateMarketVaultResult {
    pub briber: Keypair,
    pub market_vault: Address,
    pub outcome_yes_mint: Keypair,
    pub outcome_no_mint: Keypair,
    pub briber_ata: Address,
    pub market_vault_treasury: Address,
    pub outcome_yes_vault: Address,
    pub outcome_no_vault: Address,
}

pub fn run(client: &RpcClient, init: &InitializeResult) -> CreateMarketVaultResult {
    let program_id = helpers::program_id();
    let token_program = helpers::token_program();
    let system_program = helpers::system_program();

    let briber = Keypair::new();
    helpers::airdrop(client, &briber.pubkey(), 2_000_000_000);

    // Outcome mints (the briber creates them)
    let outcome_yes_mint = Keypair::new();
    let outcome_no_mint = Keypair::new();

    let (create_yes_mint_ix, init_yes_mint_ix) =
        helpers::create_and_init_mint(client, &briber.pubkey(), &outcome_yes_mint, 6);
    let (create_no_mint_ix, init_no_mint_ix) =
        helpers::create_and_init_mint(client, &briber.pubkey(), &outcome_no_mint, 6);

    // Create outcome mints tx
    let mut mint_tx = Transaction::new_with_payer(
        &[
            create_yes_mint_ix,
            init_yes_mint_ix,
            create_no_mint_ix,
            init_no_mint_ix,
        ],
        Some(&briber.pubkey()),
    );
    mint_tx.sign(
        &[&briber, &outcome_yes_mint, &outcome_no_mint],
        client.get_latest_blockhash().unwrap(),
    );
    match client.send_and_confirm_transaction(&mint_tx) {
        Ok(sig) => println!("[create_market_vault] Mints created: {}", sig),
        Err(err) => panic!("[create_market_vault] Error creating mints: {}", err),
    }

    // Derive briber's ATA for incentive mint (wsol)
    let associated_token_program = helpers::associated_token_program();
    let (briber_ata, _) = Address::find_program_address(
        &[
            briber.pubkey().as_ref(),
            token_program.as_ref(),
            init.wsol_mint.pubkey().as_ref(),
        ],
        &associated_token_program,
    );

    // Create ATA instruction (idempotent variant, discriminator = 1)
    let create_ata_ix = Instruction::new_with_bytes(
        associated_token_program,
        &[1u8], // CreateIdempotent
        vec![
            AccountMeta::new(briber.pubkey(), true),           // funding
            AccountMeta::new(briber_ata, false),               // ata
            AccountMeta::new_readonly(briber.pubkey(), false), // wallet
            AccountMeta::new_readonly(init.wsol_mint.pubkey(), false), // mint
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ],
    );

    // MintTo instruction (spl-token discriminator = 7)
    let mut mint_to_data = vec![7u8]; // MintTo
    mint_to_data.extend_from_slice(&100_000_000_000u64.to_le_bytes()); // 100 wsol
    let mint_to_ix = Instruction::new_with_bytes(
        token_program,
        &mint_to_data,
        vec![
            AccountMeta::new(init.wsol_mint.pubkey(), false), // mint
            AccountMeta::new(briber_ata, false),              // destination
            AccountMeta::new_readonly(init.authority.pubkey(), true), // mint authority
        ],
    );

    // Send create ATA + mint_to in one tx (authority signs for mint_to)
    let mut fund_tx =
        Transaction::new_with_payer(&[create_ata_ix, mint_to_ix], Some(&briber.pubkey()));
    fund_tx.sign(
        &[&briber, &init.authority],
        client.get_latest_blockhash().unwrap(),
    );
    match client.send_and_confirm_transaction(&fund_tx) {
        Ok(sig) => println!("[create_market_vault] Briber ATA funded: {}", sig),
        Err(err) => panic!("[create_market_vault] Error funding briber ATA: {}", err),
    }

    // Derive PDAs
    // market_vault uses market_counter = 1 (first market after initialize)
    let market_counter: u64 = 1;
    let (market_vault, _) = Address::find_program_address(
        &[b"market_vault", &market_counter.to_le_bytes()],
        &program_id,
    );

    let (market_vault_treasury, _) =
        Address::find_program_address(&[b"treasury", market_vault.as_ref()], &program_id);

    let (outcome_yes_vault, _) = Address::find_program_address(
        &[
            b"treasury",
            market_vault.as_ref(),
            outcome_yes_mint.pubkey().as_ref(),
        ],
        &program_id,
    );

    let (outcome_no_vault, _) = Address::find_program_address(
        &[
            b"treasury",
            market_vault.as_ref(),
            outcome_no_mint.pubkey().as_ref(),
        ],
        &program_id,
    );

    // Build create_market_vault instruction
    let ix_accounts = vec![
        AccountMeta::new(briber.pubkey(), true),
        AccountMeta::new(init.config, false),
        AccountMeta::new_readonly(init.wsol_mint.pubkey(), false), // incentive_mint
        AccountMeta::new(init.wsol_treasury, false),               // protocol treasury
        AccountMeta::new(market_vault, false),
        AccountMeta::new_readonly(outcome_yes_mint.pubkey(), false),
        AccountMeta::new_readonly(outcome_no_mint.pubkey(), false),
        AccountMeta::new(briber_ata, false),
        AccountMeta::new(market_vault_treasury, false),
        AccountMeta::new(outcome_yes_vault, false),
        AccountMeta::new(outcome_no_vault, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let mut ix_data = vec![1u8]; // discriminator
    let incentive_amount: u64 = 10_000_000_000; // 10 wsol incentive
    ix_data.extend_from_slice(&incentive_amount.to_le_bytes());

    let instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&briber.pubkey()));
    transaction.sign(&[&briber], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => println!("[create_market_vault] Transaction Signature: {}", signature),
        Err(err) => panic!("[create_market_vault] Error sending transaction: {}", err),
    }

    CreateMarketVaultResult {
        briber,
        market_vault,
        outcome_yes_mint,
        outcome_no_mint,
        briber_ata,
        market_vault_treasury,
        outcome_yes_vault,
        outcome_no_vault,
    }
}
