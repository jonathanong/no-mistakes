use oxc_ast::ast::Expression;
pub(super) fn named(text: &str) -> bool {
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
            .is_some_and(|text| named(&text)),
        _ => false,
    }
}
