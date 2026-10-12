//! Trove's domain logic: types, validation, the layout solver and ports (traits). This crate does no
//! IO and must not depend on Tauri, so it can be tested and reused on its own.

pub mod hlc;
pub mod id;
pub mod op;

pub use hlc::{Clock, ClockError, Hlc, MAX_DRIFT_MS};
pub use id::Id;
pub use op::{OP_VERSION, Operation, Patch, PatchError};
