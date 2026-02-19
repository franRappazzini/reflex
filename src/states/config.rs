use pinocchio::{error::ProgramError, Address};

use crate::{errors::ReflexError, require_eq_len};

#[repr(C)]
pub struct Config {
    pub authority: Address,
    pub market_counter: [u8; 8], // u64 - 1 byte aligned
    pub fee_bps: [u8; 2],        // u16
    pub briber_fee_bps: [u8; 2], // u16
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = size_of::<Config>();

    pub fn _load(bytes: &[u8]) -> Result<&Self, ProgramError> {
        require_eq_len!(bytes.len(), Self::LEN);

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &*(bytes.as_ptr() as *const Self) })
    }

    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        require_eq_len!(bytes.len(), Self::LEN);

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &mut *(bytes.as_mut_ptr() as *mut Self) })
    }

    pub fn set_inner(&mut self, authority: Address, fee_bps: u16, briber_fee_bps: u16, bump: u8) {
        self.authority = authority;
        self.market_counter = 0u64.to_le_bytes();
        self.fee_bps = fee_bps.to_le_bytes();
        self.briber_fee_bps = briber_fee_bps.to_le_bytes();
        self.bump = bump;
    }
}
