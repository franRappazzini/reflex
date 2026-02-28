use mollusk_svm::result::Check;

use crate::{errors::ReflexError, tests::instructions::*};

#[test]
fn test_initialize() {
    initialize_test::run_initialize();
}

#[test]
fn test_create_market_vault() {
    let mut init = initialize_test::run_initialize();
    create_market_vault_test::run_create_market_vault(&mut init);
}

#[test]
fn test_stake_outcome_tokens() {
    let mut init = initialize_test::run_initialize();
    let market = create_market_vault_test::run_create_market_vault(&mut init);
    stake_outcome_tokens_test::run_stake_outcome_tokens(&mut init, &market);
}

#[test]
fn test_unstake_outcome_tokens() {
    let mut init = initialize_test::run_initialize();
    let market = create_market_vault_test::run_create_market_vault(&mut init);
    let stake = stake_outcome_tokens_test::run_stake_outcome_tokens(&mut init, &market);
    unstake_outcome_tokens_test::run_unstake_outcome_tokens(&mut init, &stake);
}

#[test]
fn test_add_incentives() {
    let mut init = initialize_test::run_initialize();
    let market = create_market_vault_test::run_create_market_vault(&mut init);
    let stake = stake_outcome_tokens_test::run_stake_outcome_tokens(&mut init, &market);
    unstake_outcome_tokens_test::run_unstake_outcome_tokens(&mut init, &stake);
    add_incentives_test::run_add_incentives(&mut init, &market);
}

#[test]
fn test_cancel_market_fail() {
    let mut init = initialize_test::run_initialize();
    let market = create_market_vault_test::run_create_market_vault(&mut init);
    let stake = stake_outcome_tokens_test::run_stake_outcome_tokens(&mut init, &market);
    unstake_outcome_tokens_test::run_unstake_outcome_tokens(&mut init, &stake);

    let checks = [Check::err(ReflexError::MarketCanNotBeCancelled.into())];
    cancel_market_test::run_cancel_market(&mut init, &market, Some(&stake), &checks);
}

#[test]
fn test_cancel_market() {
    let mut init = initialize_test::run_initialize();
    let market = create_market_vault_test::run_create_market_vault(&mut init);

    let checks = [Check::success()];
    cancel_market_test::run_cancel_market(&mut init, &market, None, &checks);
}

#[test]
fn test_settle_market() {
    let mut init = initialize_test::run_initialize();
    let market = create_market_vault_test::run_create_market_vault(&mut init);
    settle_market_test::run_settle_market(&mut init, &market);
}

#[test]
fn test_update_config() {
    let mut init = initialize_test::run_initialize();
    update_config_test::run_update_config(&mut init);
}
