/// Ensures that two expressions have equal length, returning an `InvalidAccountSize` error if they do not.
///
/// # Arguments
///
/// * `$left` - The first expression to compare (typically a length).
/// * `$right` - The second expression to compare (typically a length).
///
/// # Errors
///
/// Returns an error of type `ReflexError::InvalidAccountSize` if the two expressions are not equal.
///
/// # Example
///
/// ```rust
/// require_eq_len!(vec1.len(), vec2.len());
/// ```
#[macro_export]
macro_rules! require_eq_len {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(ReflexError::InvalidAccountSize.into());
        }
    };
}

/// Ensures that two addresses are equal, returning an `Err(ReflexError::InvalidAddress)` if they are not.
///
/// # Arguments
///
/// * `$left` - The first address expression to compare.
/// * `$right` - The second address expression to compare.
///
/// # Errors
///
/// Returns an error if the two addresses are not equal.
///
/// # Example
///
/// ```rust
/// require_eq_address!(address1, address2);
/// ```
#[macro_export]
macro_rules! require_eq_address {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(ReflexError::InvalidAddress.into());
        }
    };
}
