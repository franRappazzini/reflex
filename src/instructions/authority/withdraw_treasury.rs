use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};
use pinocchio_token::state::TokenAccount;

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address,
    states::Config,
    utils::{Account, MintInterface},
};

pub struct WithdrawTreasury<'a> {
    accounts: WithdrawTreasuryAccounts<'a>,
}

pub struct WithdrawTreasuryAccounts<'a> {
    authority: &'a AccountView,
    config: &'a AccountView,
    mint: &'a AccountView,
    treasury: &'a AccountView,
    authority_ata: &'a AccountView,
    bump_treasury: u8,
    // token_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for WithdrawTreasuryAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config, mint, treasury, authority_ata, _token_program] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(authority)?;

        let (config_address, _) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);
        require_eq_address!(&config_address, config.address());

        let (treasury_address, bump_treasury) = Address::find_program_address(
            &[constants::TREASURY_SEED, mint.address().as_ref()],
            &crate::ID,
        );
        require_eq_address!(&treasury_address, treasury.address());

        Ok(Self {
            authority,
            config,
            mint,
            treasury,
            authority_ata,
            bump_treasury,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for WithdrawTreasury<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = WithdrawTreasuryAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> WithdrawTreasury<'a> {
    pub const DISCRIMINATOR: &'a u8 = &10;

    pub fn process(&mut self) -> ProgramResult {
        // check authority
        let config_data = self.accounts.config.try_borrow()?;
        let config = Config::load(&config_data)?;

        require_eq_address!(&config.authority, self.accounts.authority.address());

        let treasury_amount = {
            // check balance > 0
            let treasury = TokenAccount::from_account_view(self.accounts.treasury)?;
            if treasury.amount() == 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            treasury.amount()
        };

        // transfer
        let bump_binding = [self.accounts.bump_treasury];
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(self.accounts.mint.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        MintInterface::transfer_signed(
            self.accounts.treasury,
            self.accounts.authority_ata,
            self.accounts.config,
            treasury_amount,
            &seeds,
        )
    }
}
