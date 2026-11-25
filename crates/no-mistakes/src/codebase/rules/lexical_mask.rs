//! Language-aware source masking for lightweight Swift and C# rules.
//!
//! The masks retain byte offsets and line endings, so regex findings can use
//! their offsets against the original source for accurate locations.

mod common;
mod csharp;
mod swift;

pub(crate) use csharp::csharp_code_mask;
pub(crate) fn swift_code_mask(source: &str) -> String {
    swift::mask(source, false)
}

pub(crate) fn swift_declaration_mask(source: &str) -> String {
    swift::mask(source, true)
}

#[cfg(test)]
mod tests;
