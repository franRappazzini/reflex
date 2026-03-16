use pinocchio::{Address, error::ProgramError};

#[repr(C)]
pub struct Config {
    authority: [u8; 32],     // Address
    fee_bps: [u8; 2],        // u16
    briber_fee_bps: [u8; 2], // u16
    pub bump: u8,
}

impl Config {
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
    pub fn set_inner(&mut self, authority: &Address, fee_bps: u16, briber_fee_bps: u16, bump: u8) {
        self.authority = authority.to_bytes();
        self.fee_bps = fee_bps.to_le_bytes();
        self.briber_fee_bps = briber_fee_bps.to_le_bytes();
        self.bump = bump;
    }

    #[inline(always)]
    pub fn authority(&self) -> Address {
        Address::new_from_array(self.authority)
    }

    #[inline(always)]
    pub fn fee_bps(&self) -> u16 {
        u16::from_le_bytes(self.fee_bps)
    }

    #[inline(always)]
    pub fn briber_fee_bps(&self) -> u16 {
        u16::from_le_bytes(self.briber_fee_bps)
    }
}
