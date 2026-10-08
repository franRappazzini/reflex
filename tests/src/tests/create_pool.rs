use std::rc::Rc;

use anchor_client::Transaction;
use solana_signer::Signer;

use crate::utils::{
    derive_outcome_vault, prepare_mint_and_ata, IncentiveSetup, OutcomeSetup, PoolSetup,
    TestContext, HARDCODED_MARKET, INITIAL_MINT_AMOUNT, INITIAL_OUTCOME_AMOUNT,
    INITIAL_POOL_FUNDING_AMOUNT, PREDICT_PROGRAM_ID, USDC_DECIMALS,
};

pub(super) fn run(ctx: &TestContext) -> PoolSetup {
    let incentive_mint = prepare_mint_and_ata(
        &ctx.program,
        &ctx.accounts.creator.pubkey(),
        USDC_DECIMALS,
        INITIAL_MINT_AMOUNT,
    );
    let incentive_mint_address = incentive_mint.mint.pubkey();
    let yes = prepare_mint_and_ata(
        &ctx.program,
        &ctx.accounts.staker.pubkey(),
        USDC_DECIMALS,
        INITIAL_OUTCOME_AMOUNT,
    );
    let no = prepare_mint_and_ata(
        &ctx.program,
        &ctx.accounts.staker.pubkey(),
        USDC_DECIMALS,
        INITIAL_OUTCOME_AMOUNT,
    );

    let yes_setup = OutcomeSetup {
        mint: yes.mint.pubkey(),
        ata: yes.ata,
        vault: derive_outcome_vault(&ctx.program.id(), &ctx.pdas.pool, &yes.mint.pubkey()),
    };
    let no_setup = OutcomeSetup {
        mint: no.mint.pubkey(),
        ata: no.ata,
        vault: derive_outcome_vault(&ctx.program.id(), &ctx.pdas.pool, &no.mint.pubkey()),
    };

    // The NO token setup is a separate transaction to keep pool creation within packet limits.
    let mut no_setup_tx = Transaction::new_with_payer(
        &[
            no.create_account_ix,
            no.init_mint_ix,
            no.create_ata_ix,
            no.mint_to_ix,
        ],
        Some(&ctx.accounts.staker.pubkey()),
    );
    no_setup_tx.sign(
        &[
            no.mint.insecure_clone(),
            ctx.accounts.staker.insecure_clone(),
        ],
        ctx.program
            .rpc()
            .get_latest_blockhash()
            .expect("failed to get recent blockhash"),
    );
    ctx.program
        .rpc()
        .send_and_confirm_transaction(&no_setup_tx)
        .expect("failed to prepare NO outcome mint");

    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::CreatePool {
            signer: ctx.accounts.creator.pubkey(),
            config: ctx.pdas.config,
            pool: ctx.pdas.pool,
            market: HARDCODED_MARKET,
            mint: incentive_mint_address,
            yes_outcome_mint: yes_setup.mint,
            no_outcome_mint: no_setup.mint,
            signer_ata: incentive_mint.ata,
            pool_vault: ctx.pdas.pool_vault,
            pool_yes_vault: yes_setup.vault,
            pool_no_vault: no_setup.vault,
            predict_program: PREDICT_PROGRAM_ID,
            token_program: spl_token_interface::ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::CreatePool {
            amount: INITIAL_POOL_FUNDING_AMOUNT,
        })
        .instruction(incentive_mint.create_account_ix)
        .instruction(incentive_mint.init_mint_ix)
        .instruction(incentive_mint.create_ata_ix)
        .instruction(incentive_mint.mint_to_ix)
        .instruction(yes.create_account_ix)
        .instruction(yes.init_mint_ix)
        .instruction(yes.create_ata_ix)
        .instruction(yes.mint_to_ix)
        .payer(Rc::from(ctx.accounts.creator.insecure_clone()))
        .signer(incentive_mint.mint)
        .signer(yes.mint)
        .signer(ctx.accounts.staker.insecure_clone())
        .send()
        .expect("create pool transaction failed");

    println!("✅ Create pool transaction: {signature}");

    PoolSetup {
        incentive: IncentiveSetup {
            mint: incentive_mint_address,
            ata: incentive_mint.ata,
        },
        yes: yes_setup,
        no: no_setup,
    }
}
