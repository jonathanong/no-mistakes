use super::*;
use crate::codebase::postgres::embedded::helpers::program::FrozenEntry;

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
            .map(|(index, entry)| {
                let mut fields = BTreeMap::new();
                for field in &entry.fields {
                    fields.insert(field.name.clone(), Node::Text(field.sql.sql.clone()));
                }
                Arm {
                    node: Node::Object(fields),
                    choices: vec![(call.span.start as u64 | (1 << 62), index as u32)],
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
) -> Option<Vec<&'a FrozenEntry>> {
    let argument = call.arguments.get(key_index)?.as_expression()?;
    let Some(keys) = expression(visitor, argument, depth - 1) else {
        return Some(entries.iter().collect());
    };
    let mut chosen = Vec::new();
    for key in keys {
        let Node::Text(text) = &key.node else {
            return None;
        };
        let entry = entries.iter().find(|entry| entry.key == *text)?;
        if !chosen
            .iter()
            .any(|seen: &&FrozenEntry| seen.key == entry.key)
        {
            chosen.push(entry);
        }
    }
    Some(chosen)
}
