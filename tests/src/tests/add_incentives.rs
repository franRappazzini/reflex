use solana_signer::Signer;
use std::rc::Rc;

use crate::utils::{
    associated_token_address, derive_funder_pda, fetch_account, IncentiveSetup, TestContext,
};

pub(super) fn run(
    ctx: &TestContext,
    funder: &solana_keypair::Keypair,
    incentive: &IncentiveSetup,
    amount: u64,
) {
    let funder_address = funder.pubkey();
    let funder_ata =
        associated_token_address(&funder_address, &incentive.mint, &spl_token_interface::ID);
    let funder_pda = derive_funder_pda(&ctx.program.id(), &ctx.pdas.pool, &funder_address);

    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::AddIncentives {
            signer: funder_address,
            pool: ctx.pdas.pool,
            funder: funder_pda,
            market: ctx.world_random_market,
            mint: incentive.mint,
            pool_vault: ctx.pdas.pool_vault,
            signer_ata: funder_ata,
            token_program: spl_token_interface::ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::AddIncentives { amount })
        .payer(Rc::from(funder.insecure_clone()))
        .send()
        .expect("add incentives transaction failed");

    let funder_state: reflex::Funder = fetch_account(ctx, &funder_pda);
    assert_eq!(funder_state.funder, funder_address);
    assert_eq!(funder_state.amount, amount);

    println!("✅ Add incentives transaction: {signature}");
}
