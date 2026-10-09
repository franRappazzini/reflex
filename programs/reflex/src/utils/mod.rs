use anchor_lang::{prelude::ProgramError, require, Result};

use crate::error::ReflexError;

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

    require!(dt >= 0, ReflexError::Unauthorized);

    let delta = (staked as u128)
        .checked_mul(dt as u128)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    *acc = acc
        .checked_add(delta)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    *last_ts = ts;

    Ok(())
}
