use super::super::document::{Document, Kind};
use super::super::Reference;
use serde_yaml::Value;

/// Where an `if` key is a real GitHub Actions conditional.
///
/// Job and step `if` values are bare expressions. An action input named `if`
/// under `with:` is ordinary text and must not be scanned as one.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum IfScope {
    Action,
    Runs,
    /// A job mapping, or one element of `jobs.*.steps` / `runs.steps`.
    Condition,
    Steps,
    Plain,
}

fn child_scope(scope: IfScope, key: Option<&str>) -> IfScope {
    match (scope, key) {
        (IfScope::Action, Some("runs")) => IfScope::Runs,
        (IfScope::Runs | IfScope::Condition, Some("steps")) => IfScope::Steps,
        _ => IfScope::Plain,
    }
}

pub(super) fn visit(
    value: &Value,
    node: Option<usize>,
    document: &Document,
    bare: bool,
    scope: IfScope,
    refs: &mut Vec<Reference>,
) {
    match value {
        Value::String(text) => {
            let line = document.line(node);
            let mut found = super::super::references::collect(text, bare, line);
            if let Some(node) = node {
                let raw = &document.resolved(node).raw;
                let mut authored =
                    super::super::references::collect(raw, bare, document.resolved(node).line);
                for reference in &mut found {
                    if let Some(index) = authored
                        .iter()
                        .position(|actual| same_name(actual, reference))
                    {
                        reference.line = authored.remove(index).line;
                    } else {
                        // Escaped YAML can change expression spelling and line breaks.
                        // Keep unlocatable references incomplete rather than inventing a line.
                        reference.name = None;
                        reference.line = line;
                    }
                }
            }
            if node.is_none() {
                for reference in &mut found {
                    reference.name = None;
                }
            }
            refs.extend(found);
        }
        Value::Mapping(mapping) => {
            for (key, value) in mapping {
                let name = key.as_str();
                let child =
                    name.and_then(|key| node.and_then(|id| document.mapping_value(id, key)));
                let bare = scope == IfScope::Condition && name == Some("if");
                visit(value, child, document, bare, child_scope(scope, name), refs);
            }
        }
        Value::Sequence(values) => {
            let nodes = node.and_then(|id| match &document.resolved(id).kind {
                Kind::Sequence(nodes) => Some(nodes.as_slice()),
                _ => None,
            });
            let item_scope = if scope == IfScope::Steps {
                IfScope::Condition
            } else {
                IfScope::Plain
            };
            for (index, value) in values.iter().enumerate() {
                visit(
                    value,
                    nodes.and_then(|nodes| nodes.get(index)).copied(),
                    document,
                    false,
                    item_scope,
                    refs,
                );
            }
        }
        Value::Tagged(tagged) => visit(&tagged.value, node, document, bare, scope, refs),
        _ => {}
    }
}

fn same_name(left: &Reference, right: &Reference) -> bool {
    match (&left.name, &right.name) {
        (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
        (None, None) => true,
        _ => false,
    }
}
