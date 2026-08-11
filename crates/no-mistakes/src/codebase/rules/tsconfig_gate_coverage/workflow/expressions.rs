mod calls;
mod contexts;
pub(super) mod interpolation;
mod lexer;
mod literal_value;
mod syntax;
mod typed_scalar;
mod validation;

pub(crate) use calls::condition_function_call;
pub(crate) use contexts::REUSABLE_CALL_SECRET_BINDING_CONTEXTS;
pub(super) use interpolation::{
    interpolated_expression_valid, reduce_context_free_interpolations, resolve_interpolations,
    ContextFreeInterpolation,
};
pub(crate) use lexer::Function;
pub(super) use literal_value::{
    complete_literal_expression_value, invalid_literal_from_json, literal_from_json_value,
};
pub(super) use typed_scalar::typed_scalar_expression_contexts_available;
pub(super) use validation::{
    complete_expression_contexts_available, complete_expression_contexts_with_hash_files_available,
    condition_expression_contexts_available,
    interpolated_expression_contexts_and_hash_files_available,
    interpolated_expression_contexts_available,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StaticExpressionType {
    Boolean,
    Number,
    String,
    Null,
    Dynamic,
}

const MAX_CONDITION_LOGICAL_OPERATORS: usize = 255;

fn condition_tokens_within_budget(tokens: &[lexer::Token]) -> bool {
    tokens
        .iter()
        .filter(|token| matches!(token, lexer::Token::And | lexer::Token::Or))
        .take(MAX_CONDITION_LOGICAL_OPERATORS + 1)
        .count()
        <= MAX_CONDITION_LOGICAL_OPERATORS
}

fn condition_tokens(value: &str) -> Option<Vec<lexer::Token>> {
    let value = value.trim();
    let body = if value.starts_with("${{") || value.ends_with("}}") {
        value
            .strip_prefix("${{")
            .and_then(|body| body.strip_suffix("}}"))
            .map(str::trim)?
    } else {
        value
    };
    lexer::tokenize(body).filter(|tokens| condition_tokens_within_budget(tokens))
}

pub(super) fn complete_expression_type(value: &str) -> Option<StaticExpressionType> {
    let value = value.trim();
    let body = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
    syntax::parse(&lexer::tokenize(body)?)
}

pub(super) fn complete_expression_may_produce_mapping(value: &str) -> bool {
    if let Some(value) = complete_literal_expression_value(value) {
        return matches!(value, serde_yaml::Value::Mapping(_));
    }
    let value = value.trim();
    let Some(body) = value
        .strip_prefix("${{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
    else {
        return false;
    };
    lexer::tokenize(body)
        .and_then(|tokens| syntax::may_produce_mapping(&tokens))
        .unwrap_or(false)
}

pub(super) fn condition_expression_valid(value: &str) -> bool {
    condition_tokens(value).is_some_and(|tokens| syntax::parse(&tokens).is_some())
}

pub(super) fn condition_has_status_function(value: &str) -> bool {
    condition_tokens(value).is_some_and(|tokens| {
        syntax::parse(&tokens).is_some()
            && tokens.iter().any(|token| {
                matches!(
                    token,
                    lexer::Token::Function(
                        lexer::Function::Success
                            | lexer::Function::Failure
                            | lexer::Function::Always
                            | lexer::Function::Cancelled
                    )
                )
            })
    })
}

#[cfg(test)]
mod tests;
