//! Binding-aware call policy projected from the prepared canonical graph.

mod arguments;
mod config;
mod findings;
mod roots;
mod selection;

pub const RULE_ID: &str = "forbidden-calls";

pub(crate) use selection::{check_with_graph, graph_plan};

#[cfg(test)]
mod tests;

/// Declares opt-in argument demand without parsing application options twice.
pub fn argument_facts_required(config: &crate::config::v2::NoMistakesConfig) -> bool {
    config.rule_applications(RULE_ID).iter().any(|application| {
        application
            .options
            .get("targets")
            .and_then(serde_yaml::Value::as_sequence)
            .is_some_and(|targets| {
                targets.iter().any(|target| {
                    target
                        .get("whenArguments")
                        .is_some_and(|value| !value.is_null())
                })
            })
    })
}

#[cfg(test)]
mod argument_rule_tests;
