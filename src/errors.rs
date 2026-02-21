use pinocchio::error::{ProgramError, ToStr};
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum ReflexError {
    #[error("The size accounts do not match")]
    InvalidAccountSize,
    #[error("The accounts are not equals")]
    InvalidAddress,
    #[error("The outcome mint account is not valid")]
    InvalidOutcomeMint,
}

impl From<ReflexError> for ProgramError {
    fn from(e: ReflexError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

/*
    Deserialize Errors from Raw Values
    If you need to convert raw error codes (such as those from logs or cross-program invocations)
    back into your error enum, implement `TryFrom<u32>`:

    This is optional, but useful for advanced error handling and testing.
*/
impl TryFrom<u32> for ReflexError {
    type Error = ProgramError;
    fn try_from(error: u32) -> Result<Self, Self::Error> {
        match error {
            0 => Ok(ReflexError::InvalidAccountSize),
            1 => Ok(ReflexError::InvalidAddress),
            2 => Ok(ReflexError::InvalidOutcomeMint),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

/*
    Human Readable Errors
    For logging and debugging, you may want to provide a string representation of your errors.
    Implementing the ToStr trait allows you to do this:

    This step is also optional, but it can make error reporting more user-friendly.
*/
impl ToStr for ReflexError {
    fn to_str(&self) -> &'static str {
        match self {
            ReflexError::InvalidAccountSize => "Error: The size accounts do not match",
            ReflexError::InvalidAddress => "Error: The accounts are not equals",
            ReflexError::InvalidOutcomeMint => "Error: The outcome mint account is not valid",
        }
    }
}
