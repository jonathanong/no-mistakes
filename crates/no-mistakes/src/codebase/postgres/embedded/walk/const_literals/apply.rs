use super::LiteralObjects;
use crate::codebase::postgres::embedded::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlVariant};

struct Outcome {
    sql: String,
    poisoned: bool,
}

pub(super) fn apply_call(objects: &LiteralObjects, call: &mut EmbeddedSqlCall) {
    let mut poisoned = apply_optional(
        objects,
        &mut call.sql_text,
        &mut call.sql_source_positions,
        &mut call.recovered_placeholder_positions,
    );
    for variant in &mut call.variants {
        poisoned |= apply_variant(objects, variant);
    }
    if poisoned {
        // A poisoned member keeps the proved statement and falls back to the
        // placeholder the interpolation would have been. It must not become a
        // blank dynamic call, and it must not stay a confident splice.
        call.kind = EmbeddedSqlKind::Dynamic;
    }
}

pub(super) fn apply_variant(objects: &LiteralObjects, variant: &mut EmbeddedSqlVariant) -> bool {
    let outcome = rewrite(objects, &variant.sql_text);
    if outcome.sql != variant.sql_text {
        variant.sql_text = outcome.sql;
        variant.sql_source_offsets.clear();
        variant.sql_source_positions.clear();
        variant.recovered_placeholder_positions.clear();
    }
    outcome.poisoned
}

pub(super) fn apply_optional(
    objects: &LiteralObjects,
    sql: &mut Option<String>,
    source_positions: &mut Vec<crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition>,
    placeholders: &mut Vec<(u32, u32)>,
) -> bool {
    let Some(text) = sql.clone() else {
        return false;
    };
    let outcome = rewrite(objects, &text);
    if outcome.sql != text {
        let length_changed = outcome.sql.len() != text.len();
        *sql = Some(outcome.sql);
        // A literal splice only changes length. A poisoned member replaces the
        // sentinel with a placeholder, so the old source map no longer lines up.
        if outcome.poisoned {
            source_positions.clear();
        }
        if outcome.poisoned || length_changed {
            placeholders.clear();
        }
    }
    outcome.poisoned
}

fn rewrite(objects: &LiteralObjects, text: &str) -> Outcome {
    let literals = objects.literals.borrow();
    if literals.is_empty() || !text.contains('\0') {
        return Outcome {
            sql: text.to_string(),
            poisoned: false,
        };
    }
    let mut out = String::new();
    let mut rest = text;
    let mut poisoned = false;
    while let Some(start) = rest.find('\0') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let closed = after
            .find('\0')
            .map(|end| (end, &after[..end]))
            .filter(|(_, id)| id.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|(end, id)| id.parse::<usize>().ok().map(|index| (end, index)))
            .and_then(|(end, index)| literals.get(index).map(|splice| (end, splice)));
        if let Some((end, splice)) = closed {
            if objects.poisoned.contains(&splice.name) {
                out.push_str(&splice.fallback);
                poisoned = true;
            } else {
                out.push_str(&splice.literal);
            }
            rest = &after[end + 1..];
            continue;
        }
        // A cooked `'\0'` escape is user SQL, not a splice marker.
        out.push('\0');
        rest = after;
    }
    out.push_str(rest);
    Outcome { sql: out, poisoned }
}

#[cfg(test)]
mod tests;
