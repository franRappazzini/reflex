use std::rc::Rc;

use anchor_client::Transaction;
use solana_keypair::Keypair;
use solana_signer::Signer;
use spl_associated_token_account_interface::{
    address::get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token_interface::instruction::transfer_checked;

use crate::utils::{derive_funder_pda, fetch_account, IncentiveSetup, TestContext, USDC_DECIMALS};

pub(super) fn run(ctx: &TestContext, funder: &Keypair, incentive: &IncentiveSetup, amount: u64) {
    let funder_address = funder.pubkey();
    let funder_ata = get_associated_token_address(&funder_address, &incentive.mint);
    let creator = ctx.accounts.creator.pubkey();
    let creator_ata = get_associated_token_address(&creator, &incentive.mint);
    let funder_pda = derive_funder_pda(&ctx.program.id(), &ctx.pdas.pool, &funder_address);

    if funder_address != creator {
        let create_funder_ata = create_associated_token_account(
            &funder_address,
            &funder_address,
            &incentive.mint,
            &spl_token_interface::ID,
        );
        let transfer_incentives = transfer_checked(
            &spl_token_interface::ID,
            &creator_ata,
            &incentive.mint,
            &funder_ata,
            &creator,
            &[],
            amount,
            USDC_DECIMALS,
        )
        .expect("failed to create incentive token transfer instruction");
        let mut setup_tx = Transaction::new_with_payer(
            &[create_funder_ata, transfer_incentives],
            Some(&funder_address),
        );
        setup_tx.sign(
            &[
                funder.insecure_clone(),
                ctx.accounts.creator.insecure_clone(),
            ],
            ctx.program
                .rpc()
                .get_latest_blockhash()
                .expect("failed to get recent blockhash"),
        );
        ctx.program
            .rpc()
            .send_and_confirm_transaction(&setup_tx)
            .expect("failed to fund the independent funder's token account");
    }

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
