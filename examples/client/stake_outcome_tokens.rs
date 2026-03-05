use solana_address::Address;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;

use super::create_market_vault::CreateMarketVaultResult;
use super::helpers;

pub struct StakeResult {
    pub farmer: Keypair,
    pub farmer_position: Address,
    pub farmer_yes_ata: Address,
    pub market_vault: Address,
    pub yes_mint: Address,
    pub no_mint: Address,
    pub market_yes_vault: Address,
}

pub fn run(client: &RpcClient, market: &CreateMarketVaultResult) -> StakeResult {
    let program_id = helpers::program_id();
    let token_program = helpers::token_program();
    let system_program = helpers::system_program();

    let farmer = Keypair::new();
    helpers::airdrop(client, &farmer.pubkey(), 2_000_000_000);

    // Create farmer ATA for outcome_yes_mint and fund it
    let farmer_yes_ata = helpers::derive_ata(&farmer.pubkey(), &market.outcome_yes_mint.pubkey());
    let create_ata_ix = helpers::create_ata_ix(
        &farmer.pubkey(),
        &farmer.pubkey(),
        &market.outcome_yes_mint.pubkey(),
    );

    let stake_amount: u64 = 1_000_000_000;
    let mint_ix = helpers::mint_to_ix(
        &market.outcome_yes_mint.pubkey(),
        &farmer_yes_ata,
        &market.briber.pubkey(), // briber is mint authority
        stake_amount + 100_000,  // a little extra
    );

    let mut setup_tx =
        Transaction::new_with_payer(&[create_ata_ix, mint_ix], Some(&farmer.pubkey()));
    setup_tx.sign(
        &[&farmer, &market.briber],
        client.get_latest_blockhash().unwrap(),
    );
    match client.send_and_confirm_transaction(&setup_tx) {
        Ok(sig) => println!("[stake_outcome_tokens] Farmer ATA funded: {}", sig),
        Err(err) => panic!("[stake_outcome_tokens] Error funding farmer: {}", err),
    }

    // Derive farmer_position PDA
    let (farmer_position, _) = Address::find_program_address(
        &[
            b"farmer_position",
            market.market_vault.as_ref(),
            farmer.pubkey().as_ref(),
        ],
        &program_id,
    );

    let ix_accounts = vec![
        AccountMeta::new(farmer.pubkey(), true),
        AccountMeta::new(market.market_vault, false),
        AccountMeta::new(farmer_position, false),
        AccountMeta::new_readonly(market.outcome_yes_mint.pubkey(), false),
        AccountMeta::new(market.outcome_yes_vault, false),
        AccountMeta::new(farmer_yes_ata, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let mut ix_data = vec![2u8]; // discriminator = StakeOutcomeToken
    ix_data.extend_from_slice(&stake_amount.to_le_bytes());

    let instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&farmer.pubkey()));
    transaction.sign(&[&farmer], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&transaction) {
        Ok(sig) => println!("[stake_outcome_tokens] Transaction Signature: {}", sig),
        Err(err) => panic!("[stake_outcome_tokens] Error: {}", err),
    }

    StakeResult {
        farmer,
        farmer_position,
        farmer_yes_ata,
        market_vault: market.market_vault,
        yes_mint: market.outcome_yes_mint.pubkey(),
        no_mint: market.outcome_no_mint.pubkey(),
        market_yes_vault: market.outcome_yes_vault,
    }
}
