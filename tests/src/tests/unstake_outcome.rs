use std::rc::Rc;

use solana_signer::Signer;

use crate::utils::{OutcomeSetup, TestContext};

pub(super) fn run(ctx: &TestContext, outcome: &OutcomeSetup, amount: u64) -> Result<(), String> {
    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::UnstakeOutcome {
            staker: ctx.accounts.staker.pubkey(),
            pool: ctx.pdas.pool,
            market: ctx.world_random_market,
            position: ctx.pdas.position,
            outcome_mint: outcome.mint,
            pool_outcome_vault: outcome.vault,
            staker_ata: outcome.ata,
            token_program: spl_token_interface::ID,
        })
        .args(reflex::instruction::UnstakeOutcome { amount })
        .payer(Rc::from(ctx.accounts.staker.insecure_clone()))
        .send()
        .map_err(|error| error.to_string())?;

    println!("✅ Unstake Outcome transaction: {signature}");
    Ok(())
}
