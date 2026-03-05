mod claim_fees;
mod claim_rewards;
mod create_market_vault;
mod helpers;
mod initialize;
mod settle_market;
mod stake_outcome_tokens;
mod withdraw_treasury;

use solana_client::{rpc_client::RpcClient, rpc_config::CommitmentConfig};

#[tokio::main]
async fn main() {
    let rpc_url = String::from("http://localhost:8899");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    println!("=== Reflex Full Flow on Localnet ===\n");

    // 1. Initialize the program (sets up config, wsol/usdc treasuries)
    let init = initialize::run(&client);
    println!("Program initialized\n");

    // 2. Briber creates a market vault and deposits incentives
    let market = create_market_vault::run(&client, &init);
    println!("Market vault created\n");

    // 3. Farmer stakes outcome YES tokens
    let stake = stake_outcome_tokens::run(&client, &market);
    println!("Farmer staked outcome tokens\n");

    // 4. Authority settles the market (resolves YES)
    settle_market::run(&client, &init, &market);
    println!("Market settled (YES)\n");

    // 5. Farmer claims rewards + gets back staked tokens
    claim_rewards::run(&client, &init, &market, &stake);
    println!("Farmer claimed rewards\n");

    // 6. Briber claims outcome token fees from the vaults
    claim_fees::run(&client, &market);
    println!("Briber claimed fees\n");

    // 7. Authority withdraws accumulated protocol fees from wsol treasury
    withdraw_treasury::run(&client, &init);
    println!("Authority withdrew treasury\n");

    println!("=== Full flow completed successfully ===");
}
