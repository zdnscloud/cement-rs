// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A common error handling library for the Libra project.
//!
//! ## Usage
//!
//! // This crate must be imported as 'failure' in order to ensure the
//! // procedural derive macro for the `Fail` trait can function properly.
//! failure = { path = "../common/failure_ext", package = "failure_ext" }
//! // Most of the types and macros you'll need can be found in the prelude.
//! use failure::prelude::*;

pub use failure::{
    _core, bail, ensure, err_msg, format_err, AsFail, Backtrace, Causes, Compat, Context, Error,
    Fail, ResultExt, SyncFailure,
};

// Custom error handling macros are placed in the failure_macros crate. Due to
// the way intra-crate macro exports currently work, macros can't be exported
// from anywhere but the top level when they are defined in the same crate.

pub type Result<T> = ::std::result::Result<T, Error>;

#[macro_export]
macro_rules! bail_err {
    ($e:expr) => {
        return Err(From::from($e));
    };
}

/// Prelude module containing most commonly used types/macros this crate exports.
pub mod prelude {
    pub use crate::Result;
    pub use bail_err;
    pub use failure::{bail, ensure, err_msg, format_err, Error, Fail, ResultExt};
}
