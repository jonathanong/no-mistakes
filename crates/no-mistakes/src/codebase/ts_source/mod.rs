use ignore::WalkBuilder;
use oxc_ast::ast::{Expression, PropertyKey};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod facts;
pub mod jsx;

mod parser_diagnostic;
pub(crate) use parser_diagnostic::format_parse_diagnostic;

include!("discovery.rs");
include!("discovery_preserve.rs");
include!("disable_comments.rs");
include!("comment_parser.rs");
include!("comment_parser_modes.rs");
include!("syntax_helpers.rs");

#[cfg(test)]
mod comment_parser_tests;
#[cfg(test)]
mod tests;
