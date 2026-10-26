use std::rc::Rc;

use solana_signer::Signer;

use crate::utils::{OutcomeSetup, TestContext, TOKEN_2022_ID};

pub(super) fn run(ctx: &TestContext, outcome: &OutcomeSetup, amount: u64) -> Result<(), String> {
    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::StakeOutcome {
            staker: ctx.accounts.staker.pubkey(),
            pool: ctx.pdas.pool,
            market: ctx.world_random_market,
            position: ctx.pdas.position,
            outcome_mint: outcome.mint,
            pool_outcome_vault: outcome.vault,
            staker_ata: outcome.ata,
            token_program: TOKEN_2022_ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::StakeOutcome { amount })
        .payer(Rc::from(ctx.accounts.staker.insecure_clone()))
        .send()
        .map_err(|error| error.to_string())?;

    println!("✅ Stake outcome transaction: {signature}");
    Ok(())
}
