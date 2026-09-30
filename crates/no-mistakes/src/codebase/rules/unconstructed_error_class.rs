//! Exported error classes that no non-test source constructs or subclasses.
//!
//! An error class that is only ever tested with `instanceof` or narrowed with a
//! type guard is dead error handling: nothing can throw it. The rule reads the
//! prepared canonical call graph, so it follows aliases, barrels, namespace
//! imports, and `extends` chains without re-parsing.

mod analysis;
mod check;
mod config;

pub const RULE_ID: &str = "unconstructed-error-class";

pub(crate) use check::{check_with_graph, graph_plan};

#[cfg(test)]
mod tests;
