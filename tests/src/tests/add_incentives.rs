use std::rc::Rc;

use solana_signer::Signer;

use crate::utils::{IncentiveSetup, TestContext, HARDCODED_MARKET, PREDICT_PROGRAM_ID};

pub(super) fn run(ctx: &TestContext, incentive: &IncentiveSetup, amount: u64) {
    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::AddIncentives {
            signer: ctx.accounts.creator.pubkey(),
            pool: ctx.pdas.pool,
            market: HARDCODED_MARKET,
            mint: incentive.mint,
            pool_vault: ctx.pdas.pool_vault,
            signer_ata: incentive.ata,
            predict_program: PREDICT_PROGRAM_ID,
            token_program: spl_token_interface::ID,
        })
        .args(reflex::instruction::AddIncentives { amount })
        .payer(Rc::from(ctx.accounts.creator.insecure_clone()))
        .send()
        .expect("❌ add incentives transaction failed");

    println!("✅ Add Incentives transaction: {signature}");
}
