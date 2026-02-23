use crate::tests::instructions::*;

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
