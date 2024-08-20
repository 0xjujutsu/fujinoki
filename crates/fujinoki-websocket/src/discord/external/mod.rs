//! Instead of creating our own embed builders etc. we can just use existing
//! builders people are familiar with and check which one it is by whether or
//! not the serialization fails.
//!
//! Even if libraries have the same format for their data and value types, it
//! shouldn't matter because the library that it originates from is not
//! important, the values will be interpreted the same
// TODO move this to a different crate (discord-external-libs or
// discord-libraries?)
pub mod lilybird;
