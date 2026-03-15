use pinocchio::{
    Address, ProgramResult,
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};

#[repr(C)]
pub struct Market {
    briber: [u8; 32],                // Address
    incentive_mint: [u8; 32],        // Address
    outcome_yes_mint: [u8; 32],      // Address
    outcome_no_mint: [u8; 32],       // Address
    total_incentive_amount: [u8; 8], // u64
    total_yes_staked: [u8; 8],       // u64
    total_no_staked: [u8; 8],        // u64
    available_yes_fees: [u8; 8],     // u64
    available_no_fees: [u8; 8],      // u64
    creation_timestamp: [u8; 8],     // i64
    fee_bps: [u8; 2],                // u16
    status: MarketStatus,
    resolution: MarketResolution,
    bump: [u8; 1],
}

impl Market {
    pub const LEN: usize = size_of::<Self>();

    #[inline(always)]
    pub fn load(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn set_inner(
        &mut self,
        briber: &Address,
        incentive_mint: &Address,
        outcome_yes_mint: &Address,
        outcome_no_mint: &Address,
        total_incentive_amount: u64,
        fee_bps: u16,
        bump: u8,
    ) -> ProgramResult {
        self.briber = briber.to_bytes();
        self.incentive_mint = incentive_mint.to_bytes();
        self.outcome_yes_mint = outcome_yes_mint.to_bytes();
        self.outcome_no_mint = outcome_no_mint.to_bytes();
        self.total_incentive_amount = total_incentive_amount.to_le_bytes();
        self.total_yes_staked = 0u64.to_le_bytes();
        self.total_no_staked = 0u64.to_le_bytes();
        self.available_yes_fees = 0u64.to_le_bytes();
        self.available_no_fees = 0u64.to_le_bytes();
        self.creation_timestamp = Clock::get()?.unix_timestamp.to_le_bytes();
        self.fee_bps = fee_bps.to_le_bytes();
        self.status = MarketStatus::Open;
        self.resolution = MarketResolution::None;
        self.bump = [bump];

        Ok(())
    }

    #[inline(always)]
    pub fn briber(&self) -> Address {
        Address::new_from_array(self.briber)
    }

    #[inline(always)]
    pub fn incentive_mint(&self) -> Address {
        Address::new_from_array(self.incentive_mint)
    }

    #[inline(always)]
    pub fn is_open(&self) -> bool {
        matches!(self.status, MarketStatus::Open)
    }

    #[inline(always)]
    pub fn add_incentives(&mut self, amount: u64) -> ProgramResult {
        let new_amount = u64::from_le_bytes(self.total_incentive_amount)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.total_incentive_amount = new_amount.to_le_bytes();
        Ok(())
    }
}

#[repr(u8)]
pub enum MarketStatus {
    // Unopen = 0,
    Open = 0,
    Setted = 1,
}

#[repr(u8)]
pub enum MarketResolution {
    None = 0,
    Yes = 1,
    No = 2,
}
