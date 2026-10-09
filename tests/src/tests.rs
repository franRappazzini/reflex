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
use anchor_client::anchor_lang::prelude::program_pack::Pack;
use reqwest::blocking::Client;
use serde_json::json;
use solana_signer::Signer;

use crate::utils::{
    fetch_account, setup, PoolSetup, TestContext, ADDITIONAL_INCENTIVE_AMOUNT,
    INITIAL_OUTCOME_AMOUNT, INITIAL_POOL_FUNDING_AMOUNT, INITIAL_STAKE_AMOUNT,
};

#[test]
fn test_reflex_lifecycle() {
    let ctx = setup();

    initialize::run(&ctx);
    assert_initialized(&ctx);

    let pool = create_pool::run(&ctx);
    stake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT).expect("initial YES stake failed");
    stake_outcome::run(&ctx, &pool.no, INITIAL_STAKE_AMOUNT).expect("initial NO stake failed");

    advance_time(7);

    add_incentives::run(&ctx, &pool.incentive, ADDITIONAL_INCENTIVE_AMOUNT);

    stake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT / 2)
        .expect("additional YES stake before settlement failed");

    assert_pool_and_position(&ctx, &pool);

    advance_time(7);

    unstake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT / 2)
        .expect("unstake outcome YES failed");

    assert_after_unstake(&ctx, &pool);

    advance_time(7);
    settle_pool::run(&ctx, reflex::PoolStatus::Yes);

    claim_rewards::run(&ctx, &pool.incentive, &pool.yes);

    collect_fees::run(&ctx, &pool.yes);

    assert!(
        stake_outcome::run(&ctx, &pool.no, INITIAL_STAKE_AMOUNT / 2).is_err(),
        "staking should fail after the pool is settled"
    );
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
    u64::try_from(u128::from(amount) * u128::from(fee_bps) / 10_000)
        .expect("fee amount does not fit in u64")
}

fn net_stake(amount: u64, fee_bps: u16) -> u64 {
    amount - fee_amount(amount, fee_bps)
}

fn token_balance(ctx: &TestContext, address: &solana_pubkey::Pubkey) -> u64 {
    let account = ctx
        .program
        .rpc()
        .get_account(address)
        .expect("failed to fetch staker outcome token account");
    spl_token_interface::state::Account::unpack(&account.data)
        .expect("failed to decode staker outcome token account")
        .amount
}

fn advance_time(days: i64) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let absolute_timestamp = now + (days * SECONDS_PER_DAY as i64 * 1_000);

    let client = Client::new();
    let rpc_url = "http://127.0.0.1:8899";
    let params = json!({
        "absoluteTimestamp": absolute_timestamp
    });

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "surfnet_timeTravel",
        "params": [params]
    });

    let res = client
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send();

    if res.is_ok() {
        println!("Advance {} days", days)
    } else {
        println!("❌ Fail advancing days")
    }

    // let res = res.unwrap().json::<Value>().unwrap();
}
