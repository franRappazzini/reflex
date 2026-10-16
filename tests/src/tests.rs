mod add_incentives;
mod claim_rewards;
mod collect_fees;
mod create_pool;
mod initialize;
mod settle_pool;
mod stake_outcome;
mod unstake_outcome;
use std::time::{SystemTime, UNIX_EPOCH};

use anchor_client::anchor_lang::prelude::clock::SECONDS_PER_DAY;
use reflex::BPS_DENOMINATOR;
use serde_json::json;
use solana_signer::Signer;

use crate::utils::{
    derive_funder_pda, fetch_account, setup, surfnet_rpc, token_balance, PoolSetup, TestContext,
    ADDITIONAL_INCENTIVE_AMOUNT, INITIAL_OUTCOME_AMOUNT, INITIAL_POOL_FUNDING_AMOUNT,
    INITIAL_STAKE_AMOUNT,
};

#[test]
fn test_reflex_lifecycle() {
    let ctx = setup();

    initialize::run(&ctx);
    assert_initialized(&ctx);

    let pool = create_pool::run(&ctx);
    assert_funder(&ctx, &ctx.accounts.creator, INITIAL_POOL_FUNDING_AMOUNT);
    stake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT).expect("initial YES stake failed");
    stake_outcome::run(&ctx, &pool.no, INITIAL_STAKE_AMOUNT).expect("initial NO stake failed");

    advance_time(7);

    add_incentives::run(
        &ctx,
        &ctx.accounts.staker,
        &pool.incentive,
        ADDITIONAL_INCENTIVE_AMOUNT,
    );
    assert_funder(&ctx, &ctx.accounts.staker, ADDITIONAL_INCENTIVE_AMOUNT);
    assert_funder_weights(&ctx);
    assert_eq!(
        fetch_account::<reflex::Pool>(&ctx, &ctx.pdas.pool).total_incentive_amount,
        INITIAL_POOL_FUNDING_AMOUNT + ADDITIONAL_INCENTIVE_AMOUNT
    );

    stake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT / 2)
        .expect("additional YES stake before settlement failed");

    assert_pool_and_position(&ctx, &pool);

    advance_time(7);

    unstake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT / 2)
        .expect("unstake outcome YES failed");

    assert_after_unstake(&ctx, &pool);

    advance_time(7);
    settle_pool::run(&ctx, reflex::PoolStatus::Yes);

    collect_fees::run(&ctx, &ctx.accounts.creator, &pool.yes);
    collect_fees::run(&ctx, &ctx.accounts.staker, &pool.yes);
    claim_rewards::run(&ctx, &pool.incentive, &pool.yes);

    assert!(
        stake_outcome::run(&ctx, &pool.no, INITIAL_STAKE_AMOUNT / 2).is_err(),
        "staking should fail after the pool is settled"
    );
}

fn assert_funder(ctx: &TestContext, funder: &solana_keypair::Keypair, expected_amount: u64) {
    let funder_address = funder.pubkey();
    let funder_pda = derive_funder_pda(&ctx.program.id(), &ctx.pdas.pool, &funder_address);
    let state: reflex::Funder = fetch_account(ctx, &funder_pda);

    assert_eq!(state.funder, funder_address);
    assert_eq!(state.amount, expected_amount);
}

fn assert_funder_weights(ctx: &TestContext) {
    let pool: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    let creator_funder: reflex::Funder = fetch_account(ctx, &ctx.pdas.funder);
    let staker_funder_pda = derive_funder_pda(
        &ctx.program.id(),
        &ctx.pdas.pool,
        &ctx.accounts.staker.pubkey(),
    );
    let staker_funder: reflex::Funder = fetch_account(ctx, &staker_funder_pda);

    assert_eq!(
        pool.funder_weight_seconds,
        funder_weight_at(&creator_funder, pool.funder_last_update_ts)
            + funder_weight_at(&staker_funder, pool.funder_last_update_ts)
    );
    assert_eq!(
        pool.total_incentive_amount,
        creator_funder.amount + staker_funder.amount
    );
}

fn funder_weight_at(funder: &reflex::Funder, timestamp: i64) -> u128 {
    let elapsed = timestamp
        .checked_sub(funder.last_update_ts)
        .and_then(|elapsed| u128::try_from(elapsed).ok())
        .expect("pool funder timestamp precedes a funder's last update");
    funder
        .weight_seconds
        .checked_add(
            u128::from(funder.amount)
                .checked_mul(elapsed)
                .expect("funder weight overflow"),
        )
        .expect("funder weight overflow")
}

fn assert_initialized(ctx: &TestContext) {
    let config: reflex::Config = fetch_account(ctx, &ctx.pdas.config);

    assert_eq!(config.authority, ctx.accounts.authority.pubkey());
    assert_eq!(config.fee_bps, 500);
    assert_eq!(config.briber_fee_bps, 500);
}

fn assert_pool_and_position(ctx: &TestContext, setup: &PoolSetup) {
    let pool: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    let position: reflex::Position = fetch_account(ctx, &ctx.pdas.position);
    let initial_net_stake = net_stake(INITIAL_STAKE_AMOUNT, pool.fee_bps);
    let additional_net_stake = net_stake(INITIAL_STAKE_AMOUNT / 2, pool.fee_bps);

    assert_eq!(pool.creator, ctx.accounts.creator.pubkey());
    assert_eq!(pool.incentive_mint, setup.incentive.mint);
    assert_eq!(pool.yes_outcome_mint, setup.yes.mint);
    assert_eq!(pool.no_outcome_mint, setup.no.mint);
    assert_eq!(
        pool.total_incentive_amount,
        INITIAL_POOL_FUNDING_AMOUNT + ADDITIONAL_INCENTIVE_AMOUNT
    );
    assert_eq!(
        pool.total_yes_staked,
        initial_net_stake + additional_net_stake
    );
    assert_eq!(pool.total_no_staked, initial_net_stake);
    assert_eq!(
        pool.available_yes_fees,
        fee_amount(INITIAL_STAKE_AMOUNT, pool.fee_bps)
            + fee_amount(INITIAL_STAKE_AMOUNT / 2, pool.fee_bps)
    );
    assert_eq!(
        pool.available_no_fees,
        fee_amount(INITIAL_STAKE_AMOUNT, pool.fee_bps)
    );
    assert!(pool.is_open());

    assert_eq!(position.authority, ctx.accounts.staker.pubkey());
    assert_eq!(
        position.yes_staked,
        initial_net_stake + additional_net_stake
    );
    assert_eq!(position.no_staked, initial_net_stake);
}

fn assert_after_unstake(ctx: &TestContext, setup: &PoolSetup) {
    let pool: reflex::Pool = fetch_account(ctx, &ctx.pdas.pool);
    let position: reflex::Position = fetch_account(ctx, &ctx.pdas.position);
    let unstake_amount = INITIAL_STAKE_AMOUNT / 2;
    let yes_tokens_staked = INITIAL_STAKE_AMOUNT + INITIAL_STAKE_AMOUNT / 2;
    let expected_yes_staked = net_stake(INITIAL_STAKE_AMOUNT, pool.fee_bps)
        + net_stake(INITIAL_STAKE_AMOUNT / 2, pool.fee_bps)
        - unstake_amount;
    let expected_staker_balance = INITIAL_OUTCOME_AMOUNT - yes_tokens_staked + unstake_amount;

    assert_eq!(pool.total_yes_staked, expected_yes_staked);
    assert_eq!(position.yes_staked, expected_yes_staked);
    assert_eq!(
        pool.total_no_staked,
        net_stake(INITIAL_STAKE_AMOUNT, pool.fee_bps)
    );
    assert_eq!(
        pool.available_yes_fees,
        fee_amount(INITIAL_STAKE_AMOUNT, pool.fee_bps)
            + fee_amount(INITIAL_STAKE_AMOUNT / 2, pool.fee_bps)
    );
    assert_eq!(
        pool.available_no_fees,
        fee_amount(INITIAL_STAKE_AMOUNT, pool.fee_bps)
    );
    assert_eq!(pool.yes_stake_seconds, position.yes_stake_seconds);
    assert_eq!(pool.yes_last_update_ts, position.yes_last_update_ts);
    assert!(pool.yes_stake_seconds > 0);
    assert_eq!(token_balance(ctx, &setup.yes.ata), expected_staker_balance);
}

fn fee_amount(amount: u64, fee_bps: u16) -> u64 {
    u64::try_from(u128::from(amount) * u128::from(fee_bps) / BPS_DENOMINATOR as u128)
        .expect("fee amount does not fit in u64")
}

fn net_stake(amount: u64, fee_bps: u16) -> u64 {
    amount - fee_amount(amount, fee_bps)
}

fn advance_time(days: i64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let absolute_timestamp = now + (days * SECONDS_PER_DAY as i64 * 1_000);

    surfnet_rpc(
        "surfnet_timeTravel",
        json!([{"absoluteTimestamp": absolute_timestamp}]),
    )
    .expect("failed to advance Surfpool time");
    println!("Advanced time by {days} days");
}
