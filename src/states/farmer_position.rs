use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{errors::ReflexError, require_eq_len};

#[repr(C)]
pub struct FarmerPosition {
    pub farmer: Address,
    pub bump: u8,
    pub is_initialized: bool,
    pub yes_staked: [u8; 8], // u64
    pub no_staked: [u8; 8],  // u64
}

impl FarmerPosition {
    pub const LEN: usize = size_of::<Self>();

    #[inline(always)]
    pub fn _load(bytes: &[u8]) -> Result<&Self, ProgramError> {
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
    pub fn set_inner(&mut self, farmer: &AccountView, bump: u8) {
        self.farmer = farmer.address().clone();
        self.yes_staked = 0u64.to_le_bytes();
        self.no_staked = 0u64.to_le_bytes();
        self.bump = bump;
        self.is_initialized = true;
    }

    #[inline(always)]
    pub fn add_yes_staked(&mut self, amount: u64) -> ProgramResult {
        self.yes_staked = u64::from_le_bytes(self.yes_staked)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn add_no_staked(&mut self, amount: u64) -> ProgramResult {
        self.no_staked = u64::from_le_bytes(self.no_staked)
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?
            .to_le_bytes();
        Ok(())
    }
}
