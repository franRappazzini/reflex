use anchor_lang::{prelude::ProgramError, require, Result};

use crate::{error::ReflexError, BPS_DENOMINATOR};

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
