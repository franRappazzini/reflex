use std::rc::Rc;

use solana_signer::Signer;

use crate::utils::{IncentiveSetup, TestContext};

pub(super) fn run(ctx: &TestContext, incentive: &IncentiveSetup, amount: u64) {
    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::AddIncentives {
            signer: ctx.accounts.creator.pubkey(),
            pool: ctx.pdas.pool,
            market: ctx.world_random_market,
            mint: incentive.mint,
            pool_vault: ctx.pdas.pool_vault,
            signer_ata: incentive.ata,
            token_program: spl_token_interface::ID,
        })
        .args(reflex::instruction::AddIncentives { amount })
        .payer(Rc::from(ctx.accounts.creator.insecure_clone()))
        .send()
        .expect("❌ add incentives transaction failed");

    println!("✅ Add Incentives transaction: {signature}");
}
