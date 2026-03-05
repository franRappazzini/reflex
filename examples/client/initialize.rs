use solana_address::Address;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_signer::Signer;

use super::helpers;

pub struct InitializeResult {
    pub authority: Keypair,
    pub config: Address,
    pub wsol_mint: Keypair,
    pub usdc_mint: Keypair,
    pub wsol_treasury: Address,
    pub usdc_treasury: Address,
}

pub fn run(client: &RpcClient) -> InitializeResult {
    let program_id = helpers::program_id();
    let token_program = helpers::token_program();
    let system_program = helpers::system_program();

    let authority = Keypair::new();
    helpers::airdrop(client, &authority.pubkey(), 2_000_000_000);

    let wsol_mint = Keypair::new();
    let usdc_mint = Keypair::new();

    let (config, _) = Address::find_program_address(&[b"config"], &program_id);
    let (wsol_treasury, _) =
        Address::find_program_address(&[b"treasury", wsol_mint.pubkey().as_ref()], &program_id);
    let (usdc_treasury, _) =
        Address::find_program_address(&[b"treasury", usdc_mint.pubkey().as_ref()], &program_id);

    // Create and init mints
    let (create_wsol_ix, init_wsol_ix) =
        helpers::create_and_init_mint(client, &authority.pubkey(), &wsol_mint, 9);
    let (create_usdc_ix, init_usdc_ix) =
        helpers::create_and_init_mint(client, &authority.pubkey(), &usdc_mint, 6);

    // Initialize ix
    let ix_accounts = vec![
        AccountMeta::new(authority.pubkey(), true),
        AccountMeta::new(config, false),
        AccountMeta::new_readonly(wsol_mint.pubkey(), false),
        AccountMeta::new_readonly(usdc_mint.pubkey(), false),
        AccountMeta::new(wsol_treasury, false),
        AccountMeta::new(usdc_treasury, false),
        AccountMeta::new_readonly(token_program, false),
        AccountMeta::new_readonly(system_program, false),
    ];

    let fee_bps = 500u16;
    let briber_fee_bps = 500u16;
    let mut ix_data = vec![0u8];
    ix_data.extend_from_slice(&[fee_bps.to_le_bytes(), briber_fee_bps.to_le_bytes()].concat());

    let instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

    let mut transaction = Transaction::new_with_payer(
        &[
            create_wsol_ix,
            init_wsol_ix,
            create_usdc_ix,
            init_usdc_ix,
            instruction,
        ],
        Some(&authority.pubkey()),
    );
    transaction.sign(
        &[&authority, &wsol_mint, &usdc_mint],
        client.get_latest_blockhash().unwrap(),
    );

    match client.send_and_confirm_transaction(&transaction) {
        Ok(signature) => println!("[initialize] Transaction Signature: {}", signature),
        Err(err) => panic!("[initialize] Error sending transaction: {}", err),
    }

    InitializeResult {
        authority,
        config,
        wsol_mint,
        usdc_mint,
        wsol_treasury,
        usdc_treasury,
    }
}
