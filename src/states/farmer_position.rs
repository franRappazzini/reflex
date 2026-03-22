use pinocchio::{ProgramResult, error::ProgramError};

#[repr(C)]
pub struct FarmerPosition {
    yes_staked: [u8; 8], // u64
    no_staked: [u8; 8],  // u64
    pub is_initialized: bool,
    pub bump: u8,
}

impl FarmerPosition {
    pub const LEN: usize = size_of::<Self>();

    #[inline(always)]
    pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        };

        // SAFETY: everything is u8 aligned and length checked
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }

    #[inline(always)]
    pub fn set_inner(&mut self, bump: u8) {
        self.bump = bump;
        self.is_initialized = true;
    }

    #[inline(always)]
    pub fn yes_staked(&self) -> u64 {
        u64::from_le_bytes(self.yes_staked)
    }

    #[inline(always)]
    pub fn no_staked(&self) -> u64 {
        u64::from_le_bytes(self.no_staked)
    }

    #[inline(always)]
    pub fn add_yes_staked(&mut self, amount: u64) -> ProgramResult {
        let new_amount = self
            .yes_staked()
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.yes_staked = new_amount.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn add_no_staked(&mut self, amount: u64) -> ProgramResult {
        let new_amount = self
            .no_staked()
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.no_staked = new_amount.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn sub_yes_staked(&mut self, amount: u64) -> ProgramResult {
        let new_amount = self
            .yes_staked()
            .checked_sub(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.yes_staked = new_amount.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn sub_no_staked(&mut self, amount: u64) -> ProgramResult {
        let new_amount = self
            .no_staked()
            .checked_sub(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.no_staked = new_amount.to_le_bytes();
        Ok(())
    }
}
