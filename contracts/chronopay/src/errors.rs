//! ChronoPay error types for structured error handling.
//!
//! This module defines the error types used throughout the ChronoPay contract
//! to provide expressive, auditable, and composable error reporting.

use soroban_sdk::Error;

/// ChronoPay contract errors.
///
/// Each variant represents a distinct failure mode that can occur during
/// contract operations.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ChronoPayError {
    /// Caller is not authorized to perform this action.
    Unauthorized = 1,

    /// The specified address is invalid.
    InvalidAddress = 2,

    /// The time token has already been sold and cannot be transferred.
    TokenAlreadySold = 3,

    /// The time token has already been redeemed and cannot be redeemed again.
    TokenAlreadyRedeemed = 4,

    /// The time token does not exist or has not been minted.
    TokenNotFound = 5,

    /// The caller does not own the specified time token.
    TokenNotOwned = 6,

    /// The transfer operation failed.
    TransferFailed = 7,

    /// Insufficient balance for the requested operation.
    InsufficientBalance = 8,

    /// The time slot is invalid or expired.
    InvalidTimeSlot = 9,
}

impl From<ChronoPayError> for Error {
    fn from(e: ChronoPayError) -> Error {
        Error::from_contract_error(e as u32)
    }
}

impl From<&ChronoPayError> for Error {
    fn from(e: &ChronoPayError) -> Error {
        Error::from_contract_error(*e as u32)
    }
}

impl TryFrom<Error> for ChronoPayError {
    type Error = Error;

    fn try_from(e: Error) -> Result<Self, Self::Error> {
        // In newer Soroban SDK, Error wraps a Val. We can try to convert to u32 via the raw value.
        // Use the error's inner conversion - Error wraps a u32-like value in newer SDK
        let val: soroban_sdk::Val = e.into();
        let code: Result<u32, _> = val.try_into();
        match code {
            Ok(1) => Ok(ChronoPayError::Unauthorized),
            Ok(2) => Ok(ChronoPayError::InvalidAddress),
            Ok(3) => Ok(ChronoPayError::TokenAlreadySold),
            Ok(4) => Ok(ChronoPayError::TokenAlreadyRedeemed),
            Ok(5) => Ok(ChronoPayError::TokenNotFound),
            Ok(6) => Ok(ChronoPayError::TokenNotOwned),
            Ok(7) => Ok(ChronoPayError::TransferFailed),
            Ok(8) => Ok(ChronoPayError::InsufficientBalance),
            Ok(9) => Ok(ChronoPayError::InvalidTimeSlot),
            _ => Err(e),
        }
    }
}
