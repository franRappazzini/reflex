use std::rc::Rc;

use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::utils::{
    associated_token_address, derive_outcome_vault, IncentiveSetup, OutcomeSetup, PoolSetup,
    TestContext, INITIAL_POOL_FUNDING_AMOUNT,
};

pub(super) fn run(ctx: &TestContext) -> PoolSetup {
    let incentive_mint = reflex::constants::USDC_MINT_ID;
    let yes = OutcomeSetup {
        mint: ctx.world_outcome_mints.yes,
        ata: associated_token_address(
            &ctx.accounts.staker.pubkey(),
            &ctx.world_outcome_mints.yes,
            &spl_token_2022_interface::ID,
        ),
        vault: derive_outcome_vault(
            &ctx.program.id(),
            &ctx.pdas.pool,
            &ctx.world_outcome_mints.yes,
        ),
    };
    let no = OutcomeSetup {
        mint: ctx.world_outcome_mints.no,
        ata: associated_token_address(
            &ctx.accounts.staker.pubkey(),
            &ctx.world_outcome_mints.no,
            &spl_token_2022_interface::ID,
        ),
        vault: derive_outcome_vault(
            &ctx.program.id(),
            &ctx.pdas.pool,
            &ctx.world_outcome_mints.no,
        ),
    };
    let yes_receipt_mint = Keypair::new();
    let no_receipt_mint = Keypair::new();
    let incentive_ata = associated_token_address(
        &ctx.accounts.creator.pubkey(),
        &incentive_mint,
        &spl_token_interface::ID,
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
            yes_outcome_mint: yes.mint,
            no_outcome_mint: no.mint,
            yes_receipt_mint: yes_receipt_mint.pubkey(),
            no_receipt_mint: no_receipt_mint.pubkey(),
            signer_ata: incentive_ata,
            pool_vault: ctx.pdas.pool_vault,
            pool_yes_vault: yes.vault,
            pool_no_vault: no.vault,
            spl_token_program: spl_token_interface::ID,
            token_program: spl_token_2022_interface::ID,
            system_program: solana_sdk_ids::system_program::id(),
        })
        .args(reflex::instruction::CreatePool {
            amount: INITIAL_POOL_FUNDING_AMOUNT,
        })
        .payer(Rc::from(ctx.accounts.creator.insecure_clone()))
        .signer(yes_receipt_mint)
        .signer(no_receipt_mint)
        .send()
        .expect("create pool transaction failed");

    println!("✅ Create pool transaction: {signature}");

    PoolSetup {
        incentive: IncentiveSetup {
            mint: incentive_mint,
            ata: incentive_ata,
        },
        yes,
        no,
    }
}
