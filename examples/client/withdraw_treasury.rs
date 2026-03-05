use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_signer::Signer;

use super::helpers;
use super::initialize::InitializeResult;

pub fn run(client: &RpcClient, init: &InitializeResult) {
    let program_id = helpers::program_id();
    let token_program = helpers::token_program();

    // Create authority ATA for wsol (protocol fees land here)
    let authority_wsol_ata =
        helpers::derive_ata(&init.authority.pubkey(), &init.wsol_mint.pubkey());

    let mut ata_tx = Transaction::new_with_payer(
        &[helpers::create_ata_ix(
            &init.authority.pubkey(),
            &init.authority.pubkey(),
            &init.wsol_mint.pubkey(),
        )],
        Some(&init.authority.pubkey()),
    );
    ata_tx.sign(&[&init.authority], client.get_latest_blockhash().unwrap());
    match client.send_and_confirm_transaction(&ata_tx) {
        Ok(sig) => println!("[withdraw_treasury] Authority ATA created: {}", sig),
        Err(err) => panic!("[withdraw_treasury] Error creating ATA: {}", err),
    }

    // Accounts order:
    // authority, config, mint, treasury, authority_ata, token_program
    let ix_accounts = vec![
        AccountMeta::new(init.authority.pubkey(), true),
        AccountMeta::new(init.config, false),
        AccountMeta::new_readonly(init.wsol_mint.pubkey(), false),
        AccountMeta::new(init.wsol_treasury, false),
        AccountMeta::new(authority_wsol_ata, false),
        AccountMeta::new_readonly(token_program, false),
    ];

    let instruction =
        Instruction::new_with_bytes(program_id, &[10u8], ix_accounts); // discriminator = WithdrawTreasury

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&init.authority.pubkey()));
    transaction.sign(&[&init.authority], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&transaction) {
        Ok(sig) => println!("[withdraw_treasury] Transaction Signature: {}", sig),
        Err(err) => panic!("[withdraw_treasury] Error: {}", err),
    }
}
