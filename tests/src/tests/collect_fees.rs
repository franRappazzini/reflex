use std::rc::Rc;

use anchor_client::anchor_lang::prelude::program_pack::Pack;
use anchor_client::CommitmentConfig;
use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address;

use crate::utils::{
    fetch_account, OutcomeSetup, TestContext, HARDCODED_MARKET, PREDICT_PROGRAM_ID,
};

pub(super) fn run(ctx: &TestContext, outcome: &OutcomeSetup) {
    let pool_before: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    let (expected_fees, other_side_fees) = if pool_before.is_yes() {
        (
            pool_before.available_yes_fees,
            pool_before.available_no_fees,
        )
    } else if pool_before.is_no() {
        (
            pool_before.available_no_fees,
            pool_before.available_yes_fees,
        )
    } else {
        panic!("pool must be settled before collecting fees");
    };
    assert!(expected_fees > 0, "no fees accrued for the winning outcome");

    let creator = ctx.accounts.creator.pubkey();
    let creator_ata = get_associated_token_address(&creator, &outcome.mint);
    let creator_balance_before = token_balance(ctx, &creator_ata);

    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::CollectFees {
            creator,
            pool: ctx.pdas.pool,
            market: HARDCODED_MARKET,
            outcome_mint: outcome.mint,
            pool_outcome_vault: outcome.vault,
            creator_ata,
            predict_program: PREDICT_PROGRAM_ID,
            associated_token_program: spl_associated_token_account_interface::program::ID,
            token_program: spl_token_interface::ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::CollectFees {})
        .payer(Rc::from(ctx.accounts.creator.insecure_clone()))
        .send()
        .expect("collect fees transaction failed");

    let creator_balance_after = token_balance(ctx, &creator_ata);
    assert_eq!(
        creator_balance_after - creator_balance_before,
        expected_fees,
        "creator should receive accrued fees for the winning outcome"
    );

    let pool_after: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    if pool_before.is_yes() {
        assert_eq!(pool_after.available_yes_fees, 0);
        assert_eq!(pool_after.available_no_fees, other_side_fees);
    } else {
        assert_eq!(pool_after.available_no_fees, 0);
        assert_eq!(pool_after.available_yes_fees, other_side_fees);
    }

    println!("✅ Collect Fees transaction: {signature}");
}

fn token_balance(ctx: &TestContext, address: &solana_pubkey::Pubkey) -> u64 {
    let account = ctx
        .program
        .rpc()
        .get_account_with_commitment(address, CommitmentConfig::confirmed())
        .expect("failed to fetch creator token account")
        .value;

    account.map_or(0, |account| {
        spl_token_interface::state::Account::unpack(&account.data)
            .expect("failed to decode creator token account")
            .amount
    })
}
