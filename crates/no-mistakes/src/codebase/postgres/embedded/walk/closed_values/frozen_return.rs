use super::*;
use crate::codebase::postgres::embedded::helpers::program::FrozenEntry;

type KeyChoices = Vec<(u64, u32)>;
type Chosen<'a> = (&'a FrozenEntry, KeyChoices);

/// One object arm per frozen-record entry, sharing a choice across fields.
pub(super) fn arms(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    name: &str,
    depth: u8,
) -> Option<Vec<Arm>> {
    let (key_index, entries) = visitor.functions.frozen_object(name)?;
    let selected = selected(visitor, call, key_index, entries, depth)?;
    Some(
        selected
            .into_iter()
            .enumerate()
            .map(|(index, (entry, key_choices))| {
                let mut fields = BTreeMap::new();
                for field in &entry.fields {
                    fields.insert(field.name.clone(), Node::Text(field.sql.sql.clone()));
                }
                let mut choices = vec![(
                    call.span.start as u64 | super::HELPER_OBJECT_CHOICE,
                    index as u32,
                )];
                for choice in key_choices {
                    if !choices.contains(&choice) {
                        choices.push(choice);
                    }
                }
                Arm {
                    node: Node::Object(fields),
                    choices,
                    root: None,
                    provenance: None,
                    type_annotation: false,
                }
            })
            .collect(),
    )
}

fn selected<'a>(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    key_index: usize,
    entries: &'a [FrozenEntry],
    depth: u8,
) -> Option<Vec<Chosen<'a>>> {
    let argument = call.arguments.get(key_index)?.as_expression()?;
    let Some(keys) = expression(visitor, argument, depth - 1) else {
        if entries.len() > super::MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
        return Some(entries.iter().map(|entry| (entry, Vec::new())).collect());
    };
    choose(keys, entries)
}

fn choose(keys: Vec<Arm>, entries: &[FrozenEntry]) -> Option<Vec<Chosen<'_>>> {
    let mut chosen: Vec<Chosen<'_>> = Vec::new();
    for key in keys {
        let Node::Text(text) = &key.node else {
            return None;
        };
        // A missing member drops the selection. A subset is not exhaustive.
        let entry = entries.iter().find(|entry| entry.key == *text)?;
        if chosen
            .iter()
            .any(|(seen, choices)| seen.key == entry.key && *choices == key.choices)
        {
            continue;
        }
        chosen.push((entry, key.choices));
    }
    if chosen.is_empty() || chosen.len() > super::MAX_EMBEDDED_SQL_VARIANTS {
        return None;
    }
    Some(chosen)
}

#[cfg(test)]
#[path = "frozen_return_tests.rs"]
mod tests;
