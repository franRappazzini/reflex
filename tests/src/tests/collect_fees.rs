use std::rc::Rc;

use anchor_client::CommitmentConfig;
use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::utils::{
    associated_token_address, derive_funder_pda, fetch_account, token_balance, OutcomeSetup,
    TestContext,
};

pub(super) fn run(ctx: &TestContext, signer: &Keypair, outcome: &OutcomeSetup) {
    let pool_before: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    let (available_fees, other_side_fees) = if pool_before.is_yes() {
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

    let signer_address = signer.pubkey();
    let funder_pda = derive_funder_pda(&ctx.program.id(), &ctx.pdas.pool, &signer_address);
    let funder: reflex::Funder = fetch_account(ctx, &funder_pda);
    let elapsed = pool_before
        .resolution_timestamp
        .checked_sub(funder.last_update_ts)
        .expect("resolution timestamp precedes funder update");
    let funder_weight_seconds = funder
        .weight_seconds
        .checked_add(
            u128::from(funder.amount)
                .checked_mul(u128::try_from(elapsed).expect("negative elapsed time"))
                .expect("funder weight overflow"),
        )
        .expect("funder weight overflow");
    let expected_fees = u64::try_from(
        u128::from(available_fees)
            .checked_mul(funder_weight_seconds)
            .expect("fee share overflow")
            .checked_div(pool_before.funder_weight_seconds)
            .expect("pool has no funder weight"),
    )
    .expect("fee share does not fit in u64");
    assert!(
        expected_fees > 0,
        "funder's winning-side fee share should be positive"
    );

    let signer_ata = associated_token_address(
        &signer_address,
        &outcome.mint,
        &spl_token_2022_interface::ID,
    );
    let signer_balance_before = token_balance(ctx, &signer_ata);

    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::CollectFees {
            signer: signer_address,
            pool: ctx.pdas.pool,
            funder: funder_pda,
            market: ctx.world_random_market,
            outcome_mint: outcome.mint,
            pool_outcome_vault: outcome.vault,
            signer_ata,
            associated_token_program: spl_associated_token_account_interface::program::ID,
            token_program: spl_token_2022_interface::ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::CollectFees {})
        .payer(Rc::from(signer.insecure_clone()))
        .send()
        .expect("collect fees transaction failed");

    assert_eq!(
        token_balance(ctx, &signer_ata) - signer_balance_before,
        expected_fees,
        "funder should receive its pro-rata share of winning outcome fees"
    );

    let pool_after: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    let remaining_fees = available_fees - expected_fees;
    if pool_before.is_yes() {
        assert_eq!(pool_after.available_yes_fees, remaining_fees);
        assert_eq!(pool_after.available_no_fees, other_side_fees);
    } else {
        assert_eq!(pool_after.available_no_fees, remaining_fees);
        assert_eq!(pool_after.available_yes_fees, other_side_fees);
    }
    assert_eq!(
        pool_after.funder_weight_seconds,
        pool_before.funder_weight_seconds - funder_weight_seconds
    );

    let closed_funder = ctx
        .program
        .rpc()
        .get_account_with_commitment(&funder_pda, CommitmentConfig::confirmed())
        .expect("failed to check whether funder account was closed")
        .value;
    assert!(
        closed_funder.is_none(),
        "collected funder account should close"
    );

    println!("Collect fees transaction: {signature}");
}
