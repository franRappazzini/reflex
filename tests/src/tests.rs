mod create_pool;
mod initialize;
mod stake_outcome;
use std::time::{SystemTime, UNIX_EPOCH};

use anchor_client::anchor_lang::prelude::clock::SECONDS_PER_DAY;
use reqwest::blocking::Client;
use serde_json::json;
use solana_signer::Signer;

use crate::utils::{
    fetch_account, setup, PoolSetup, TestContext, INITIAL_POOL_FUNDING_AMOUNT, INITIAL_STAKE_AMOUNT,
};

#[test]
fn test_reflex_lifecycle() {
    let ctx = setup();

    initialize::run(&ctx);
    assert_initialized(&ctx);

    let pool = create_pool::run(&ctx);
    stake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT);
    stake_outcome::run(&ctx, &pool.no, INITIAL_STAKE_AMOUNT);

    advance_time(7);

    stake_outcome::run(&ctx, &pool.yes, INITIAL_STAKE_AMOUNT / 2);

    assert_pool_and_position(&ctx, &pool);
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

    println!("yes secs: {}", pool.yes_stake_seconds);
    println!("no secs: {}", pool.no_stake_seconds);

    assert_eq!(pool.creator, ctx.accounts.creator.pubkey());
    assert_eq!(pool.incentive_mint, setup.incentive_mint);
    assert_eq!(pool.yes_outcome_mint, setup.yes.mint);
    assert_eq!(pool.no_outcome_mint, setup.no.mint);
    assert_eq!(pool.total_incentive_amount, INITIAL_POOL_FUNDING_AMOUNT);
    assert_eq!(
        pool.total_yes_staked,
        INITIAL_STAKE_AMOUNT + (INITIAL_STAKE_AMOUNT / 2)
    );
    assert_eq!(pool.total_no_staked, INITIAL_STAKE_AMOUNT);
    assert!(pool.is_open());

    assert_eq!(position.authority, ctx.accounts.staker.pubkey());
    assert_eq!(
        position.yes_staked,
        INITIAL_STAKE_AMOUNT + (INITIAL_STAKE_AMOUNT / 2)
    );
    assert_eq!(position.no_staked, INITIAL_STAKE_AMOUNT);
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
