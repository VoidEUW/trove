//! The one error the UI sees (ADR 4). `trove-core` and `trove-store` return their own `thiserror`
//! types; only this module maps them to `AppError { code, message }`. The UI shows text for `code`,
//! in the user's language (ADR 15); `message` is English text for logs and bug reports.

use serde::Serialize;
use trove_core::id::IdError;
use trove_core::{ClockError, ItemError, PatchError};
use trove_store::StoreError;
use ts_rs::TS;

/// What went wrong, in terms the UI can show text for. Every code needs a key in
/// `apps/desktop/src/lib/i18n/en.json` named by the UI's code-to-key table (`api/errors.ts`), which is
/// typed, so a missing one fails `npm run check`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ErrorCode {
    /// The database or the data folder could not be read or written.
    Storage,
    /// The data was saved by a newer Trove.
    DatabaseTooNew,
    /// The request was refused as not valid.
    Invalid,
    /// An item was saved without a name.
    NameRequired,
    /// An item's quantity is not a whole number from 1.
    QuantityInvalid,
    /// The kind chosen for an item does not exist.
    UnknownKind,
    /// A bug or damaged data: nothing the user can fix.
    Internal,
}

/// The error of every command: `Result<T, AppError>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS, thiserror::Error)]
#[error("{code:?}: {message}")]
#[ts(export)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        // No wildcard: a new store error must be given a code here.
        let code = match &error {
            StoreError::DatabaseTooNew { .. } => ErrorCode::DatabaseTooNew,
            StoreError::Sqlite(_) | StoreError::Migration(_) | StoreError::Io { .. } => {
                ErrorCode::Storage
            }
            StoreError::Item(ItemError::NameRequired) => ErrorCode::NameRequired,
            StoreError::Item(ItemError::QuantityInvalid(_)) => ErrorCode::QuantityInvalid,
            StoreError::UnknownKind(_) => ErrorCode::UnknownKind,
            StoreError::Projection(_) => ErrorCode::Invalid,
            StoreError::Id(_)
            | StoreError::Clock(_)
            | StoreError::Json(_)
            | StoreError::UnknownEntity(_)
            | StoreError::DuplicateOperation(_)
            | StoreError::UnsupportedOpVersion { .. } => ErrorCode::Internal,
        };
        Self::new(code, error.to_string())
    }
}

impl From<ItemError> for AppError {
    fn from(error: ItemError) -> Self {
        let code = match error {
            ItemError::NameRequired => ErrorCode::NameRequired,
            ItemError::QuantityInvalid(_) => ErrorCode::QuantityInvalid,
        };
        Self::new(code, error.to_string())
    }
}

impl From<PatchError> for AppError {
    fn from(error: PatchError) -> Self {
        Self::new(ErrorCode::Invalid, error.to_string())
    }
}

impl From<IdError> for AppError {
    fn from(error: IdError) -> Self {
        Self::new(ErrorCode::Invalid, error.to_string())
    }
}

impl From<ClockError> for AppError {
    fn from(error: ClockError) -> Self {
        Self::new(ErrorCode::Internal, error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use trove_core::Id;

    use super::*;

    #[test]
    fn serialises_as_a_code_and_a_message() {
        let error = AppError::new(ErrorCode::DatabaseTooNew, "too new");
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            json!({ "code": "database_too_new", "message": "too new" })
        );
    }

    #[test]
    fn store_errors_get_the_code_the_user_can_act_on() {
        let too_new = StoreError::DatabaseTooNew {
            found: 9,
            supported: 1,
        };
        let error = AppError::from(too_new);
        assert_eq!(error.code, ErrorCode::DatabaseTooNew);
        assert!(error.message.contains("version 9"), "{}", error.message);

        let io = StoreError::Io {
            path: "trove.db".into(),
            source: std::io::Error::other("disk full"),
        };
        assert_eq!(AppError::from(io).code, ErrorCode::Storage);
        assert_eq!(
            AppError::from(StoreError::Projection("no".into())).code,
            ErrorCode::Invalid
        );
        assert_eq!(
            AppError::from(StoreError::UnknownEntity("x".into())).code,
            ErrorCode::Internal
        );
        assert_eq!(
            AppError::from(StoreError::DuplicateOperation(Id::new(1))).code,
            ErrorCode::Internal
        );
    }

    #[test]
    fn item_errors_keep_their_own_code_and_text() {
        let error = AppError::from(ItemError::NameRequired);
        assert_eq!(error.code, ErrorCode::NameRequired);
        assert_eq!(error.message, ItemError::NameRequired.to_string());

        let invalid = ItemError::QuantityInvalid(-1.0);
        let error = AppError::from(invalid);
        assert_eq!(error.code, ErrorCode::QuantityInvalid);
        assert_eq!(error.message, invalid.to_string());
    }

    #[test]
    fn an_internal_error_carries_its_message() {
        let error = AppError::internal("the task panicked");
        assert_eq!(error.code, ErrorCode::Internal);
        assert_eq!(error.message, "the task panicked");
    }

    #[test]
    fn core_errors_are_mapped_here_and_nowhere_else() {
        assert_eq!(
            AppError::from(PatchError::EmptyField).code,
            ErrorCode::Invalid
        );
        assert_eq!(
            "nope"
                .parse::<Id>()
                .map_err(AppError::from)
                .unwrap_err()
                .code,
            ErrorCode::Invalid
        );
        assert_eq!(
            AppError::from(ClockError::CounterOverflow).code,
            ErrorCode::Internal
        );
    }
}
