use std::rc::Rc;

use solana_signer::Signer;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

use crate::utils::{
    derive_outcome_vault, fetch_market_outcome_mints, set_token_account_balance, IncentiveSetup,
    OutcomeSetup, PoolSetup, TestContext, INITIAL_MINT_AMOUNT, INITIAL_OUTCOME_AMOUNT,
    INITIAL_POOL_FUNDING_AMOUNT, TOKEN_2022_ID,
};

pub(super) fn run(ctx: &TestContext) -> PoolSetup {
    let incentive_mint = reflex::USDC_MINT_ID;
    let incentive_ata = get_associated_token_address_with_program_id(
        &ctx.accounts.creator.pubkey(),
        &incentive_mint,
        &spl_token_interface::ID,
    );
    set_token_account_balance(
        &ctx.accounts.creator.pubkey(),
        &incentive_mint,
        INITIAL_MINT_AMOUNT,
        &spl_token_interface::ID,
    );

    let (yes_mint, no_mint) = fetch_market_outcome_mints(ctx);
    let yes_setup = OutcomeSetup {
        mint: yes_mint,
        ata: get_associated_token_address_with_program_id(
            &ctx.accounts.staker.pubkey(),
            &yes_mint,
            &TOKEN_2022_ID,
        ),
        vault: derive_outcome_vault(&ctx.program.id(), &ctx.pdas.pool, &yes_mint),
    };
    let no_setup = OutcomeSetup {
        mint: no_mint,
        ata: get_associated_token_address_with_program_id(
            &ctx.accounts.staker.pubkey(),
            &no_mint,
            &TOKEN_2022_ID,
        ),
        vault: derive_outcome_vault(&ctx.program.id(), &ctx.pdas.pool, &no_mint),
    };

    set_token_account_balance(
        &ctx.accounts.staker.pubkey(),
        &yes_mint,
        INITIAL_OUTCOME_AMOUNT,
        &TOKEN_2022_ID,
    );
    set_token_account_balance(
        &ctx.accounts.staker.pubkey(),
        &no_mint,
        INITIAL_OUTCOME_AMOUNT,
        &TOKEN_2022_ID,
    );

    let signature = ctx
        .program
        .request()
        .accounts(reflex::accounts::CreatePool {
            signer: ctx.accounts.creator.pubkey(),
            config: ctx.pdas.config,
            pool: ctx.pdas.pool,
            funder: ctx.pdas.funder,
            market: ctx.world_random_market,
            mint: incentive_mint,
            yes_outcome_mint: yes_setup.mint,
            no_outcome_mint: no_setup.mint,
            signer_ata: incentive_ata,
            pool_vault: ctx.pdas.pool_vault,
            pool_yes_vault: yes_setup.vault,
            pool_no_vault: no_setup.vault,
            spl_token_program: spl_token_interface::ID,
            token_program: TOKEN_2022_ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::CreatePool {
            amount: INITIAL_POOL_FUNDING_AMOUNT,
        })
        .payer(Rc::from(ctx.accounts.creator.insecure_clone()))
        .send()
        .expect("create pool transaction failed");

    println!("✅ Create pool transaction: {signature}");

    PoolSetup {
        incentive: IncentiveSetup {
            mint: incentive_mint,
            ata: incentive_ata,
        },
        yes: yes_setup,
        no: no_setup,
    }
}
