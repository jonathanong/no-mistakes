use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, TemplateLiteral};
pub(in crate::codebase::postgres::query_annotation::heads) fn named(text: &str) -> bool {
    // Inspect only the FIRST block comment; a later named comment cannot repair
    // an empty first one. Decoded/cooked text determines runtime string bytes.
    text.trim_start()
        .strip_prefix("/*")
        .and_then(|tail| tail.find("*/").map(|end| !tail[..end].trim().is_empty()))
        .unwrap_or(false)
}
pub(super) fn literal(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::StringLiteral(value) => named(&crate::ast::js_literal(value)),
        Expression::TemplateLiteral(value) => value
            .quasis
            .first()
            .filter(|part| part.value.cooked.is_some())
            .map(crate::ast::template_quasi_text)
            .is_some_and(|text| named(&text) || interpolated_name(value, &text)),
        _ => false,
    }
}

fn interpolated_name(template: &TemplateLiteral<'_>, first: &str) -> bool {
    let Some(open) = first.trim_start().strip_prefix("/*") else {
        return false;
    };
    if open.contains("*/") {
        return false;
    }
    let mut previous_star = open.ends_with('*');
    let mut has_name = !open.trim().is_empty();
    for (expression, part) in template
        .expressions
        .iter()
        .zip(template.quasis.iter().skip(1))
    {
        if let Some(choices) = known_choices(expression) {
            let mut possible_star = false;
            for choice in &choices {
                let mut ends_star = previous_star;
                if closes_comment(choice, &mut ends_star) {
                    return false;
                }
                possible_star |= ends_star;
            }
            previous_star = possible_star;
            has_name |= choices.iter().all(|choice| !choice.trim().is_empty());
        } else {
            previous_star = false;
            has_name = true;
        }
        let text = crate::ast::template_quasi_text(part);
        if let Some(end) = text.find("*/") {
            return !closes_comment(&text[..end], &mut previous_star)
                && (has_name || !text[..end].trim().is_empty());
        }
        if closes_comment(&text, &mut previous_star) {
            return false;
        }
        has_name |= !text.trim().is_empty();
    }
    false
}

fn known_choices(expression: &Expression<'_>) -> Option<Vec<String>> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(value) => Some(vec![crate::ast::js_literal(value).into_owned()]),
        _ => super::super::finite_comments::plain_ternary(expression),
    }
}

fn closes_comment(text: &str, previous_star: &mut bool) -> bool {
    let closes = (*previous_star && text.starts_with('/')) || text.contains("*/");
    if !text.is_empty() {
        *previous_star = text.ends_with('*');
    }
    closes
}
