mod lexer;
mod parser;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StaticExpressionType {
    Boolean,
    Number,
    String,
    Null,
    Dynamic,
}

pub(super) fn complete_expression_type(value: &str) -> Option<StaticExpressionType> {
    let value = value.trim();
    let body = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
    parser::parse(&lexer::tokenize(body)?)
}

pub(super) fn condition_expression_valid(value: &str) -> bool {
    let value = value.trim();
    if value.starts_with("${{") || value.ends_with("}}") {
        complete_expression_type(value).is_some()
    } else {
        lexer::tokenize(value)
            .and_then(|tokens| parser::parse(&tokens))
            .is_some()
    }
}

#[cfg(test)]
mod tests;
