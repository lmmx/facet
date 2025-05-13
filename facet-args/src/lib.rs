#![warn(missing_docs)]
#![warn(clippy::std_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

extern crate alloc;

/// Errors raised when CLI arguments are not parsed or otherwise fail during reflection
pub mod error;
/// CLI argument format implementation for facet-deserialize
pub mod format;

use error::ArgsError;
use facet_core::Facet;
use format::from_slice_with_format;

/// Parses command-line arguments
///
/// This is a wrapper around `from_slice_with_format` that uses the CliFormat
/// implementation of the Format trait.
pub fn from_slice<'input, 'facet, T>(s: &'facet [&'input str]) -> Result<T, ArgsError>
where
    T: Facet<'facet>,
    'input: 'facet,
{
    from_slice_with_format(s)
}
