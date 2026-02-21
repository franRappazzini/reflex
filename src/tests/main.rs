use crate::tests::instructions::*;

#[test]
fn test_initialize() {
    let _init = initialize::run_initialize();
}

#[test]
fn test_create_market_vault() {
    let mut init = initialize::run_initialize();
    let _market = create_market_vault::run_create_market_vault(&mut init);
}

#[test]
fn test_stake_outcome_tokens() {
    let mut init = initialize::run_initialize();
    let market = create_market_vault::run_create_market_vault(&mut init);
    stake_outcome_tokens::run_stake_outcome_tokens(&mut init, &market);
}
