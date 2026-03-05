use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_signer::Signer;

use super::create_market_vault::CreateMarketVaultResult;
use super::helpers;

pub fn run(client: &RpcClient, market: &CreateMarketVaultResult) {
    let program_id = helpers::program_id();
    let token_program = helpers::token_program();
    let system_program = helpers::system_program();

    // Create briber ATAs for both outcome tokens (fees land here)
    let briber_outcome_yes =
        helpers::derive_ata(&market.briber.pubkey(), &market.outcome_yes_mint.pubkey());
    let briber_outcome_no =
        helpers::derive_ata(&market.briber.pubkey(), &market.outcome_no_mint.pubkey());

    let mut ata_tx = Transaction::new_with_payer(
        &[
            helpers::create_ata_ix(
                &market.briber.pubkey(),
                &market.briber.pubkey(),
                &market.outcome_yes_mint.pubkey(),
            ),
            helpers::create_ata_ix(
                &market.briber.pubkey(),
                &market.briber.pubkey(),
                &market.outcome_no_mint.pubkey(),
            ),
        ],
        Some(&market.briber.pubkey()),
    );
    ata_tx.sign(&[&market.briber], client.get_latest_blockhash().unwrap());
    match client.send_and_confirm_transaction(&ata_tx) {
        Ok(sig) => println!("[claim_fees] Briber ATAs created: {}", sig),
        Err(err) => panic!("[claim_fees] Error creating ATAs: {}", err),
    }

    // Accounts order:
    // briber, market, outcome_yes_mint, outcome_no_mint,
    // briber_outcome_yes, briber_outcome_no,
    // outcome_yes_vault, outcome_no_vault, token_program, system_program
    let ix_accounts = vec![
        AccountMeta::new(market.briber.pubkey(), true),
        AccountMeta::new(market.market_vault, false),
        AccountMeta::new_readonly(market.outcome_yes_mint.pubkey(), false),
        AccountMeta::new_readonly(market.outcome_no_mint.pubkey(), false),
        AccountMeta::new(briber_outcome_yes, false),
        AccountMeta::new(briber_outcome_no, false),
        AccountMeta::new(market.outcome_yes_vault, false),
        AccountMeta::new(market.outcome_no_vault, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let instruction = Instruction::new_with_bytes(program_id, &[9u8], ix_accounts); // discriminator = ClaimFees

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&market.briber.pubkey()));
    transaction.sign(&[&market.briber], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&transaction) {
        Ok(sig) => println!("[claim_fees] Transaction Signature: {}", sig),
        Err(err) => panic!("[claim_fees] Error: {}", err),
    }
}
