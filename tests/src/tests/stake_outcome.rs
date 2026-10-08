use std::rc::Rc;

use solana_signer::Signer;

use crate::utils::{OutcomeSetup, TestContext, HARDCODED_MARKET, PREDICT_PROGRAM_ID};

pub(super) fn run(ctx: &TestContext, outcome: &OutcomeSetup, amount: u64) {
    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::StakeOutcome {
            staker: ctx.accounts.staker.pubkey(),
            pool: ctx.pdas.pool,
            market: HARDCODED_MARKET,
            position: ctx.pdas.position,
            outcome_mint: outcome.mint,
            pool_outcome_vault: outcome.vault,
            staker_ata: outcome.ata,
            predict_program: PREDICT_PROGRAM_ID,
            token_program: spl_token_interface::ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::StakeOutcome { amount })
        .payer(Rc::from(ctx.accounts.staker.insecure_clone()))
        .send()
        .expect("stake outcome transaction failed");

    println!("✅ Stake outcome transaction: {signature}");
}
