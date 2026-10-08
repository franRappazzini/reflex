use std::rc::Rc;

use solana_signer::Signer;

use crate::utils::{TestContext, HARDCODED_MARKET};

pub(super) fn run(ctx: &TestContext, decision: reflex::PoolStatus) {
    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::SettlePool {
            authority: ctx.accounts.authority.pubkey(),
            config: ctx.pdas.config,
            pool: ctx.pdas.pool,
            market: HARDCODED_MARKET,
        })
        .args(reflex::instruction::SettlePool { decision })
        .payer(Rc::from(ctx.accounts.authority.insecure_clone()))
        .send()
        .expect("❌ settle market transaction failed");

    println!("✅ Settle Market transaction: {signature}");
}
