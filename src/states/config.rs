use pinocchio::{error::ProgramError, Address, ProgramResult};

use crate::{errors::ReflexError, require_eq_len};

#[repr(C)]
pub struct Config {
    pub authority: Address,
    market_counter: [u8; 8], // u64 - 1 byte aligned
    fee_bps: [u8; 2],        // u16
    briber_fee_bps: [u8; 2], // u16
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = size_of::<Config>();

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
    pub fn market_counter(&self) -> u64 {
        u64::from_le_bytes(self.market_counter)
    }
    #[inline(always)]
    pub fn fee_bps(&self) -> u16 {
        u16::from_le_bytes(self.fee_bps)
    }
    #[inline(always)]
    pub fn briber_fee_bps(&self) -> u16 {
        u16::from_le_bytes(self.briber_fee_bps)
    }

    #[inline(always)]
    pub fn set_inner(&mut self, authority: Address, fee_bps: u16, briber_fee_bps: u16, bump: u8) {
        self.authority = authority;
        self.market_counter = 0u64.to_le_bytes();
        self.fee_bps = fee_bps.to_le_bytes();
        self.briber_fee_bps = briber_fee_bps.to_le_bytes();
        self.bump = bump;
    }

    #[inline(always)]
    pub fn set_fee_bps(&mut self, fee_bps: u16) {
        self.fee_bps = fee_bps.to_le_bytes();
    }

    #[inline(always)]
    pub fn set_briber_fee_bps(&mut self, briber_fee_bps: u16) {
        self.briber_fee_bps = briber_fee_bps.to_le_bytes();
    }

    #[inline(always)]
    pub fn add_market_counter(&mut self) -> ProgramResult {
        self.market_counter = u64::from_le_bytes(
            self.market_counter
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        )
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .to_le_bytes();

        Ok(())
    }
}
