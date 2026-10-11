//! A deliberately small summary for a helper that mutates and returns one
//! typed SQLStatement argument. We collect the suffix once, then attach it to
//! a verified caller argument; no helper body is executed at a call site.

use super::super::chain;
use super::shadows::TagShadows;
use crate::codebase::postgres::embedded::placeholders::{
    count_placeholders, renumber_placeholders,
};
use crate::codebase::postgres::embedded::{source_positions, EmbeddedSqlSourcePosition};
use crate::codebase::ts_source::byte_offset_to_line;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, FormalParameters, FunctionBody, Statement, TSType,
    TSTypeName, VariableDeclarationKind,
};
use std::collections::HashSet;

pub(super) struct ParameterBuilder {
    pub(super) parameter_index: usize,
    pub(super) parameter_count: usize,
    pub(super) suffix: String,
    pub(super) fragments: Vec<ParameterFragment>,
}

pub(super) struct ParameterFragment {
    pub(super) sql: String,
    pub(super) line: u32,
    pub(super) positions: Vec<EmbeddedSqlSourcePosition>,
}

impl ParameterBuilder {
    pub(super) fn accepts_call(&self, call: &CallExpression<'_>) -> bool {
        if call.optional || call.arguments.len() != self.parameter_count {
            return false;
        }
        let Some(argument) = call.arguments[self.parameter_index].as_expression() else {
            return false;
        };
        // A scalar argument is evaluated before the helper runs. If the
        // builder is an existing binding, a call in another argument could
        // mutate it first; keep that execution order opaque.
        if !matches!(unwrap_ts_wrappers(argument), Expression::Identifier(_)) {
            return true;
        }
        call.arguments.iter().enumerate().all(|(index, argument)| {
            if index == self.parameter_index {
                return true;
            }
            let Some(expr) = argument.as_expression() else {
                return false;
            };
            matches!(
                unwrap_ts_wrappers(expr),
                Expression::Identifier(_)
                    | Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NullLiteral(_)
            )
        })
    }
}

pub(super) fn summarize(
    params: &FormalParameters<'_>,
    body: &FunctionBody<'_>,
    types: &HashSet<String>,
    shadows: &TagShadows,
    source: &str,
) -> Option<ParameterBuilder> {
    if params.rest.is_some() {
        return None;
    }
    let mut builder = None;
    for (index, param) in params.items.iter().enumerate() {
        let Some(annotation) = &param.type_annotation else {
            continue;
        };
        let TSType::TSTypeReference(reference) = &annotation.type_annotation else {
            continue;
        };
        let TSTypeName::IdentifierReference(ty) = &reference.type_name else {
            continue;
        };
        if !types.contains(ty.name.as_str()) {
            continue;
        }
        let BindingPattern::BindingIdentifier(ident) = &param.pattern else {
            return None;
        };
        if builder.replace((index, ident.name.as_str())).is_some() {
            return None;
        }
    }
    let (parameter_index, parameter_name) = builder?;
    let mut aliases = HashSet::from([parameter_name]);
    let mut suffix = String::new();
    let mut fragments = Vec::new();
    let mut statements = body.statements.iter().peekable();
    while let Some(statement) = statements.next() {
        match statement {
            Statement::EmptyStatement(_) => {}
            Statement::VariableDeclaration(declaration)
                if matches!(
                    declaration.kind,
                    VariableDeclarationKind::Const | VariableDeclarationKind::Let
                ) =>
            {
                for declarator in &declaration.declarations {
                    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
                        return None;
                    };
                    if aliases.contains(ident.name.as_str()) {
                        return None;
                    }
                    let init = declarator.init.as_ref()?;
                    let appended = alias_or_append(init, &aliases, params, shadows, source)?;
                    if let Some(fragment) = appended {
                        suffix.push_str(&renumber_placeholders(
                            &fragment.sql,
                            count_placeholders(&suffix),
                        ));
                        fragments.push(fragment);
                    }
                    aliases.insert(ident.name.as_str());
                }
            }
            Statement::ExpressionStatement(statement) => {
                let appended =
                    alias_or_append(&statement.expression, &aliases, params, shadows, source)??;
                suffix.push_str(&renumber_placeholders(
                    &appended.sql,
                    count_placeholders(&suffix),
                ));
                fragments.push(appended);
            }
            Statement::ReturnStatement(ret) if statements.peek().is_none() => {
                let returned = ret.argument.as_ref()?;
                match unwrap_ts_wrappers(returned) {
                    Expression::Identifier(ident) if aliases.contains(ident.name.as_str()) => {}
                    Expression::CallExpression(call) => {
                        let fragment = append(call, &aliases, params, shadows, source)?;
                        suffix.push_str(&renumber_placeholders(
                            &fragment.sql,
                            count_placeholders(&suffix),
                        ));
                        fragments.push(fragment);
                    }
                    _ => return None,
                }
                return Some(ParameterBuilder {
                    parameter_index,
                    parameter_count: params.items.len(),
                    suffix,
                    fragments,
                });
            }
            _ => return None,
        }
    }
    None
}

fn alias_or_append(
    expr: &Expression<'_>,
    aliases: &HashSet<&str>,
    params: &FormalParameters<'_>,
    shadows: &TagShadows,
    source: &str,
) -> Option<Option<ParameterFragment>> {
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(ident) if aliases.contains(ident.name.as_str()) => Some(None),
        Expression::CallExpression(call) => {
            append(call, aliases, params, shadows, source).map(Some)
        }
        _ => None,
    }
}

fn append(
    call: &CallExpression<'_>,
    aliases: &HashSet<&str>,
    params: &FormalParameters<'_>,
    shadows: &TagShadows,
    source: &str,
) -> Option<ParameterFragment> {
    if call.arguments.len() != 1 || call.optional {
        return None;
    }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.property.name != "append" || member.optional {
        return None;
    }
    let Expression::Identifier(receiver) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    if !aliases.contains(receiver.name.as_str()) {
        return None;
    }
    let arg = call.arguments.first()?.as_expression()?;
    let mut names = HashSet::new();
    for param in &params.items {
        super::super::for_each_bound_name(&param.pattern, &mut |name| {
            names.insert(name.to_string());
        });
    }
    if let Some(rest) = &params.rest {
        super::super::for_each_bound_name(&rest.rest.argument, &mut |name| {
            names.insert(name.to_string());
        });
    }
    if !static_append_fragment(arg, &names, aliases) {
        return None;
    }
    // The fragment shape check permits only direct scalar placeholders. It
    // rejects builder aliases and calls that could mutate the builder.
    let sql = chain::resolve_expr(
        arg,
        super::MAX_RESOLVE_DEPTH,
        &mut |_, _, _| None,
        &mut |tag| names.contains(tag) || aliases.contains(tag) || shadows.contains(tag),
        shadows.imported_tags(),
    )?;
    let line = byte_offset_to_line(source, call.span.start as usize);
    let positions = source_positions::for_expression(arg, source, call.span.start as usize, line);
    Some(ParameterFragment {
        sql,
        line,
        positions,
    })
}

fn static_append_fragment(
    expr: &Expression<'_>,
    scalar_params: &HashSet<String>,
    aliases: &HashSet<&str>,
) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        Expression::TaggedTemplateExpression(tagged) => {
            tagged
                .quasi
                .expressions
                .iter()
                .all(|value| match unwrap_ts_wrappers(value) {
                    Expression::Identifier(ident) => {
                        scalar_params.contains(ident.name.as_str())
                            && !aliases.contains(ident.name.as_str())
                    }
                    Expression::StringLiteral(_)
                    | Expression::NumericLiteral(_)
                    | Expression::BooleanLiteral(_)
                    | Expression::NullLiteral(_) => true,
                    _ => false,
                })
        }
        _ => false,
    }
}
