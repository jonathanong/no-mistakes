use super::{is_trusted_tag_name, TagShadows};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Class, Expression, Function};

/// A `function sql()` / `function String()` shadows that name unless it looks
/// like an in-file tagged-template implementation. Zero-arg helpers, including
/// `return process.env.SQL`, must not leave a configured import or `String.raw`
/// trusted under that spelling. The spelling itself is not a trusted tag.
pub(super) fn record_function_tag_shadow(function: &Function<'_>, shadows: &mut TagShadows) {
    let Some(id) = function.id.as_ref() else {
        return;
    };
    if !is_trusted_tag_name(id.name.as_str()) {
        return;
    }
    if !looks_like_tag_implementation(function) {
        shadows.names.insert(id.name.to_string());
    }
}

pub(crate) fn looks_like_tag_implementation(function: &Function<'_>) -> bool {
    let has_params = !function.params.items.is_empty() || function.params.rest.is_some();
    has_params && !returns_static_sql(function)
}

pub(super) fn record_class_tag_shadow(class: &Class<'_>, shadows: &mut TagShadows) {
    let Some(id) = class.id.as_ref() else {
        return;
    };
    if is_trusted_tag_name(id.name.as_str()) {
        shadows.names.insert(id.name.to_string());
    }
}

fn returns_static_sql(function: &Function<'_>) -> bool {
    let Some(body) = function.body.as_ref() else {
        return false;
    };
    let [oxc_ast::ast::Statement::ReturnStatement(ret)] = body.statements.as_slice() else {
        return false;
    };
    let Some(argument) = ret.argument.as_ref() else {
        return false;
    };
    matches!(
        unwrap_ts_wrappers(argument),
        Expression::StringLiteral(_)
            | Expression::TemplateLiteral(_)
            | Expression::TaggedTemplateExpression(_)
    )
}
