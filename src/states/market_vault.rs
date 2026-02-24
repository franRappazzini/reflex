use pinocchio::{error::ProgramError, Address, ProgramResult};

use crate::{errors::ReflexError, require_eq_len};

#[repr(C)]
pub struct MarketVault {
    pub briber: Address,
    // pub kalshi_market_id: u64,
    pub outcome_yes_mint: Address,
    pub outcome_no_mint: Address,
    pub incentive_mint: Address,
    // pub creation_timestamp: i64,
    id: [u8; 8],               // u64
    total_incentives: [u8; 8], // u64
    total_yes_staked: [u8; 8], // u64
    total_no_staked: [u8; 8],  // u64
    total_yes_fees: [u8; 8],   // u64
    total_no_fees: [u8; 8],    // u64
    fee_bps: [u8; 2],          // u16
    pub status: MarketVaultStatus,
    pub market_resolution: MarketVaultResolution,
    pub fees_claimed: bool,
    pub bump: u8,
}

impl MarketVault {
    pub const LEN: usize = size_of::<Self>();

    #[inline(always)]
    pub fn load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        require_eq_len!(bytes.len(), Self::LEN);

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &*(bytes.as_ptr() as *const Self) })
    }

    #[inline(always)]
    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        require_eq_len!(bytes.len(), Self::LEN);

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn fee_bps(&self) -> u16 {
        u16::from_le_bytes(self.fee_bps)
    }

    #[inline(always)]
    pub fn total_yes_staked(&self) -> u64 {
        u64::from_le_bytes(self.total_yes_staked)
    }

    #[inline(always)]
    pub fn total_no_staked(&self) -> u64 {
        u64::from_le_bytes(self.total_no_staked)
    }

    #[inline(always)]
    pub fn total_yes_fees(&self) -> u64 {
        u64::from_le_bytes(self.total_yes_fees)
    }

    #[inline(always)]
    pub fn total_no_fees(&self) -> u64 {
        u64::from_le_bytes(self.total_no_fees)
    }

    #[inline(always)]
    pub fn total_incentives(&self) -> u64 {
        u64::from_le_bytes(self.total_incentives)
    }

    #[inline(always)]
    pub fn set_inner(
        &mut self,
        id: u64,
        briber: &Address,
        outcome_yes_mint: &Address,
        outcome_no_mint: &Address,
        incentive_mint: &Address,
        fee_bps: u16,
        bump: u8,
        initial_incentive_amount: u64,
    ) {
        self.id = id.to_le_bytes();
        self.briber = briber.clone();
        self.outcome_yes_mint = outcome_yes_mint.clone();
        self.outcome_no_mint = outcome_no_mint.clone();
        self.incentive_mint = incentive_mint.clone();
        self.fee_bps = fee_bps.to_le_bytes();
        self.bump = bump;
        self.fees_claimed = false;
        self.total_incentives = initial_incentive_amount.to_le_bytes();
        self.status = MarketVaultStatus::Open;
        self.market_resolution = MarketVaultResolution::None;
        self.total_yes_staked = 0u64.to_le_bytes();
        self.total_no_staked = 0u64.to_le_bytes();
        self.total_yes_fees = 0u64.to_le_bytes();
        self.total_no_fees = 0u64.to_le_bytes();
    }

    #[inline(always)]
    pub fn add_total_yes_staked(&mut self, amount: u64) -> ProgramResult {
        self.total_yes_staked = u64::from_le_bytes(self.total_yes_staked)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn add_total_no_staked(&mut self, amount: u64) -> ProgramResult {
        self.total_no_staked = u64::from_le_bytes(self.total_no_staked)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn sub_total_yes_staked(&mut self, amount: u64) -> ProgramResult {
        self.total_yes_staked = u64::from_le_bytes(self.total_yes_staked)
            .checked_sub(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn sub_total_no_staked(&mut self, amount: u64) -> ProgramResult {
        self.total_no_staked = u64::from_le_bytes(self.total_no_staked)
            .checked_sub(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn add_total_yes_fees(&mut self, amount: u64) -> ProgramResult {
        self.total_yes_fees = u64::from_le_bytes(self.total_yes_fees)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn add_total_no_fees(&mut self, amount: u64) -> ProgramResult {
        self.total_no_fees = u64::from_le_bytes(self.total_no_fees)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn add_incentives(&mut self, amount: u64) -> ProgramResult {
        self.total_incentives = u64::from_le_bytes(self.total_incentives)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn is_settled(&self) -> bool {
        self.status == MarketVaultStatus::Settled
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum MarketVaultStatus {
    UnOpen = 0,
    Open = 1,
    Settled = 2,
}

#[repr(u8)]
pub enum MarketVaultResolution {
    None = 0,
    Yes = 1,
    No = 2,
}
