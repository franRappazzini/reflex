use anchor_lang::{
    prelude::{
        Account, CpiContext, InterfaceAccount, Program, ProgramError, Rent, Signer, SolanaSysvar,
        System,
    },
    require, system_program, AnchorDeserialize, Result, ToAccountInfo,
};
use anchor_spl::{
    token_2022::{
        spl_token_2022::{
            extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
            state::Mint as Mint2022,
        },
        Token2022,
    },
    token_interface::{
        spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint,
        TokenMetadataInitialize,
    },
};

use crate::{error::ReflexError, Pool, BPS_DENOMINATOR};

/// Returns a net amount plus the exact fee that should be charged on the given input.
///
/// The fee is the normalized percentage of the amount using the pool's basis-point fee.
/// This keeps the fee logic consistent across stake and payout calculations and applies
/// the usual floor rounding that is expected for on-chain token math.
pub fn apply_fee(amount: u64, fee_bps: u16) -> Result<(u64, u64)> {
    let fee = (u128::from(amount))
        .checked_mul(u128::from(fee_bps))
        .and_then(|v| v.checked_div(u128::from(BPS_DENOMINATOR)))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let net_amount = amount
        .checked_sub(fee)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok((net_amount, fee))
}

/// Normalizes a proportional share using floor division.
///
/// `amount * numerator / denominator` is the standard pattern for redistributing an
/// on-chain balance across weighted stake-time shares while keeping the result in the
/// smallest token units.
pub fn normalized_share(amount: u64, numerator: u128, denominator: u128) -> Result<u64> {
    require!(denominator > 0, ReflexError::InvalidAmount);

    let share = u128::from(amount)
        .checked_mul(numerator)
        .and_then(|v| v.checked_div(denominator))
        .and_then(|v| u64::try_from(v).ok())
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(share)
}

/// Accrues `staked` over the elapsed time and updates the accumulator and timestamp.
///
/// Adds `staked * (ts - *last_ts)` to `acc`, then sets `*last_ts` to `ts`.
/// Returns [`ReflexError::Unauthorized`] if `ts` is earlier than `*last_ts`,
/// or [`ProgramError::ArithmeticOverflow`] if the time difference or arithmetic overflows.
///
/// # Arguments
///
/// * `staked` - Amount accrued per unit of elapsed time.
/// * `acc` - Accumulator to update.
/// * `last_ts` - Previous timestamp; updated to `ts` on success.
/// * `ts` - Current timestamp.
pub fn accrue(staked: u64, acc: &mut u128, last_ts: &mut i64, ts: i64) -> Result<()> {
    let dt = ts
        .checked_sub(*last_ts)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    require!(dt >= 0, ReflexError::InvalidTimestamp);

    let delta = u128::from(staked)
        .checked_mul(dt as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    *acc = acc
        .checked_add(delta)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    *last_ts = ts;

    Ok(())
}

pub fn fetch_metadata(mint: &InterfaceAccount<Mint>) -> Result<TokenMetadata> {
    let mint_account_info = mint.to_account_info();
    let mint_data = mint_account_info.try_borrow_data()?;

    // unpack mint state along with its extensions
    let mint_state = StateWithExtensions::<Mint2022>::unpack(&mint_data)?;
    let metadata: Result<TokenMetadata> =
        if let Ok(extension_bytes) = mint_state.get_extension_bytes::<TokenMetadata>() {
            // manually deserialize the borsh serialized byte slice
            TokenMetadata::try_from_slice(extension_bytes)
                .map_err(|_| ReflexError::NoDeserializeExtension.into())
        } else {
            Err(ReflexError::NoDeserializeExtension.into())
        };

    let metadata = metadata?;

    Ok(metadata)
}

pub fn initialize_metadata<'info>(
    mint: &InterfaceAccount<'info, Mint>,
    metadata: &TokenMetadata,
    signer: &Signer<'info>,
    pool: &Account<'info, Pool>,
    token_program: &Program<'info, Token2022>,
    system_program: &Program<'info, System>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let name = format!("R-{}", metadata.name);
    let symbol = format!("R-{}", metadata.symbol);
    let uri = &metadata.uri;
    let token_metadata = TokenMetadata {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        ..Default::default()
    };

    // calculate the space need for the mint account with the desired extensions
    let space =
        ExtensionType::try_calculate_account_len::<Mint2022>(&[ExtensionType::MetadataPointer])
            .unwrap();

    let meta_data_space = token_metadata.tlv_size_of().unwrap();

    let lamports = Rent::get()?.minimum_balance(space + meta_data_space);

    // transfer additional lamports to mint account
    system_program::transfer(
        CpiContext::new(
            *system_program.key,
            system_program::Transfer {
                from: signer.to_account_info(),
                to: mint.to_account_info(),
            },
        ),
        lamports,
    )?;

    // initialize token metadata
    token_metadata_initialize(
        CpiContext::new_with_signer(
            *token_program.key,
            TokenMetadataInitialize {
                program_id: token_program.to_account_info(),
                mint: mint.to_account_info(),
                metadata: mint.to_account_info(),
                mint_authority: pool.to_account_info(),
                update_authority: pool.to_account_info(),
            },
            signer_seeds,
        ),
        name,
        symbol,
        uri.clone(),
    )
}
