use pinocchio::error::ProgramError;

pub fn fee_calculation(amount: u64, fee_bps: u16) -> Result<u64, ProgramError> {
    (amount as u128)
        .checked_mul(fee_bps as u128)
        .and_then(|v| v.checked_div(10_000))
        .and_then(|v| v.try_into().ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}
