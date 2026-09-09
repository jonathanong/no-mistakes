//! Binding-aware call policy projected from the prepared canonical graph.

mod config;
mod findings;
mod roots;
mod selection;

pub const RULE_ID: &str = "forbidden-calls";

pub(crate) use selection::{check_with_graph, graph_plan};

#[cfg(test)]
mod tests;
