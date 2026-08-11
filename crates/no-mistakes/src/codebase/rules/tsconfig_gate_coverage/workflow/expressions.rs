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
    interpolated_expression_valid, reduce_context_free_interpolations, ContextFreeInterpolation,
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

pub(super) fn complete_expression_type(value: &str) -> Option<StaticExpressionType> {
    let value = value.trim();
    let body = value.strip_prefix("${{")?.strip_suffix("}}")?.trim();
    syntax::parse(&lexer::tokenize(body)?)
}

pub(super) fn complete_expression_may_produce_mapping(value: &str) -> bool {
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
    let value = value.trim();
    if value.starts_with("${{") || value.ends_with("}}") {
        complete_expression_type(value).is_some()
    } else {
        lexer::tokenize(value)
            .and_then(|tokens| syntax::parse(&tokens))
            .is_some()
    }
}

pub(super) fn condition_has_status_function(value: &str) -> bool {
    let value = value.trim();
    let body = if value.starts_with("${{") || value.ends_with("}}") {
        value
            .strip_prefix("${{")
            .and_then(|body| body.strip_suffix("}}"))
            .map(str::trim)
    } else {
        Some(value)
    };
    body.and_then(lexer::tokenize).is_some_and(|tokens| {
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
