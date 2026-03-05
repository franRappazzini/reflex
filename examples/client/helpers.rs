use solana_address::Address;
use solana_client::rpc_client::RpcClient;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_sdk::program_pack::Pack;
use solana_signer::Signer;
use solana_system_interface::instruction as system_instruction;
use spl_token::{instruction::initialize_mint2, state::Mint};

use std::str::FromStr;

pub const PROGRAM_ID: &str = "E8ApH1v8MJMgjQ8dDFyuaS95u7iW64cWhDBa8ENZ34E7";

pub fn program_id() -> Address {
    Address::from_str(PROGRAM_ID).unwrap()
}

pub fn token_program() -> Address {
    Address::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
}

pub fn system_program() -> Address {
    Address::from_str("11111111111111111111111111111111").unwrap()
}

pub fn associated_token_program() -> Address {
    Address::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap()
}

pub fn airdrop(client: &RpcClient, pubkey: &Address, amount: u64) {
    let signature = client
        .request_airdrop(pubkey, amount)
        .expect("Failed to request airdrop");

    loop {
        let confirmed = client.confirm_transaction(&signature).unwrap();
        if confirmed {
            break;
        }
    }
}

pub fn create_and_init_mint(
    client: &RpcClient,
    payer: &Address,
    mint: &Keypair,
    decimals: u8,
) -> (
    solana_instruction::Instruction,
    solana_instruction::Instruction,
) {
    let token_program = token_program();
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .unwrap();

    let create_ix = system_instruction::create_account(
        payer,
        &mint.pubkey(),
        mint_rent,
        Mint::LEN as u64,
        &token_program,
    );

    let init_ix = initialize_mint2(&token_program, &mint.pubkey(), payer, None, decimals).unwrap();

    (create_ix, init_ix)
}

/// Derives the Associated Token Account address for (owner, mint)
pub fn derive_ata(owner: &Address, mint: &Address) -> Address {
    let (ata, _) = Address::find_program_address(
        &[owner.as_ref(), token_program().as_ref(), mint.as_ref()],
        &associated_token_program(),
    );
    ata
}

/// Builds a CreateIdempotent ATA instruction (no external crate needed)
pub fn create_ata_ix(funder: &Address, owner: &Address, mint: &Address) -> Instruction {
    let ata = derive_ata(owner, mint);
    Instruction::new_with_bytes(
        associated_token_program(),
        &[1u8], // CreateIdempotent
        vec![
            AccountMeta::new(*funder, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(system_program(), false),
            AccountMeta::new_readonly(token_program(), false),
        ],
    )
}

/// Builds a raw SPL Token MintTo instruction (discriminant = 7)
pub fn mint_to_ix(mint: &Address, destination: &Address, authority: &Address, amount: u64) -> Instruction {
    let mut data = vec![7u8];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction::new_with_bytes(
        token_program(),
        &data,
        vec![
            AccountMeta::new(*mint, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(*authority, true),
        ],
    )
}
