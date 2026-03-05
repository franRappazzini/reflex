use solana_address::Address;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_response::transaction::Transaction;
use solana_instruction::{AccountMeta, Instruction};
use solana_signer::Signer;

use super::create_market_vault::CreateMarketVaultResult;
use super::helpers;
use super::initialize::InitializeResult;

pub struct SettleResult {
    pub market_vault: Address,
}

pub fn run(
    client: &RpcClient,
    init: &InitializeResult,
    market: &CreateMarketVaultResult,
) -> SettleResult {
    let program_id = helpers::program_id();

    // authority, config, market  (discriminator=6, resolution=YES=1)
    let ix_accounts = vec![
        AccountMeta::new(init.authority.pubkey(), true),
        AccountMeta::new(init.config, false),
        AccountMeta::new(market.market_vault, false),
    ];

    let ix_data = vec![6u8, 1u8]; // discriminator=6, resolution=YES

    let instruction = Instruction::new_with_bytes(program_id, &ix_data, ix_accounts);

    let mut transaction =
        Transaction::new_with_payer(&[instruction], Some(&init.authority.pubkey()));
    transaction.sign(&[&init.authority], client.get_latest_blockhash().unwrap());

    match client.send_and_confirm_transaction(&transaction) {
        Ok(sig) => println!("[settle_market] Transaction Signature: {}", sig),
        Err(err) => panic!("[settle_market] Error: {}", err),
    }

    SettleResult {
        market_vault: market.market_vault,
    }
}
