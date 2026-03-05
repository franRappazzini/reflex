use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_signer::Signer;

use super::create_market_vault::CreateMarketVaultResult;
use super::helpers;
use super::initialize::InitializeResult;
use super::stake_outcome_tokens::StakeResult;

pub fn run(
    client: &RpcClient,
    init: &InitializeResult,
    market: &CreateMarketVaultResult,
    stake: &StakeResult,
) {
    let program_id = helpers::program_id();
    let token_program = helpers::token_program();
    let system_program = helpers::system_program();
    let associated_token_program = helpers::associated_token_program();

    // Create farmer ATAs for incentive (wsol) and outcome_no (both start empty)
    let farmer_incentive_ata =
        helpers::derive_ata(&stake.farmer.pubkey(), &init.wsol_mint.pubkey());
    let farmer_outcome_no_ata = helpers::derive_ata(&stake.farmer.pubkey(), &stake.no_mint);

    let mut ata_tx = Transaction::new_with_payer(
        &[
            helpers::create_ata_ix(
                &stake.farmer.pubkey(),
                &stake.farmer.pubkey(),
                &init.wsol_mint.pubkey(),
            ),
            helpers::create_ata_ix(
                &stake.farmer.pubkey(),
                &stake.farmer.pubkey(),
                &stake.no_mint,
            ),
        ],
        Some(&stake.farmer.pubkey()),
    );
    ata_tx.sign(&[&stake.farmer], client.get_latest_blockhash().unwrap());
    match client.send_and_confirm_transaction(&ata_tx) {
        Ok(sig) => println!("[claim_rewards] Farmer ATAs created: {}", sig),
        Err(err) => panic!("[claim_rewards] Error creating ATAs: {}", err),
    }

    // Accounts order:
    // farmer, market, farmer_position, incentive_mint, outcome_yes_mint, outcome_no_mint,
    // farmer_incentive_ata, farmer_outcome_yes_ata, farmer_outcome_no_ata,
    // market_vault_treasury (incentive vault), outcome_yes_vault, outcome_no_vault,
    // associated_token_program, token_program, system_program
    let ix_accounts = vec![
        AccountMeta::new(stake.farmer.pubkey(), true),
        AccountMeta::new(stake.market_vault, false),
        AccountMeta::new(stake.farmer_position, false),
        AccountMeta::new_readonly(init.wsol_mint.pubkey(), false),
        AccountMeta::new_readonly(stake.yes_mint, false),
        AccountMeta::new_readonly(stake.no_mint, false),
        AccountMeta::new(farmer_incentive_ata, false),
        AccountMeta::new(stake.farmer_yes_ata, false),
        AccountMeta::new(farmer_outcome_no_ata, false),
        AccountMeta::new(market.market_vault_treasury, false),
        AccountMeta::new(stake.market_yes_vault, false),
        AccountMeta::new(market.outcome_no_vault, false),
        AccountMeta::new_readonly(associated_token_program, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &[8u8], ix_accounts); // discriminator = ClaimRewards

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&stake.farmer.pubkey()));
    transaction.sign(&[&stake.farmer], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&transaction) {
        Ok(sig) => println!("[claim_rewards] Transaction Signature: {}", sig),
        Err(err) => panic!("[claim_rewards] Error: {}", err),
    }
}
