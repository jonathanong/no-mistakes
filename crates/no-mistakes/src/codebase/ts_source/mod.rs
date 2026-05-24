use ignore::WalkBuilder;
use oxc::ast::ast::{Expression, PropertyKey};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod facts;
pub mod jsx;

include!("discovery.rs");
include!("disable_comments.rs");
include!("comment_parser.rs");
include!("syntax_helpers.rs");

#[cfg(test)]
mod tests;
