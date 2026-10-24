use super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::EmbeddedSqlKind;
use super::compose::static_fragment;
use super::functions::provenance::RecoveredSql;
use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(in crate::codebase::postgres::embedded) mod positions;

pub(crate) fn apply_append(visitor: &mut ScopeVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return;
    };
    if member.property.name != "append" {
        return;
    }
    let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else {
        return;
    };
    apply_append_to(visitor, ident.name.as_str(), call);
}

/// Appends `call`'s argument to the builder `name`; `call` is one `.append`
/// link, whose receiver may be an earlier link of the same fluent chain.
pub(crate) fn apply_append_to(
    visitor: &mut ScopeVisitor<'_>,
    name: &str,
    call: &CallExpression<'_>,
) {
    let Some(arg) = first_static_arg(call, visitor) else {
        visitor.mark_dynamic_keep_known_statement(name);
        return;
    };
    // Loops may append a runtime-unknown number of times. Nested functions
    // that mutate an outer binding are walked even when they never run.
    // Sequential function-local mutations still compose. Conditional static
    // fragments keep recovered SQL but mark Dynamic so INSERT cannot pass
    // a branch-only ORDER BY as if it always ran.
    if visitor.loop_depth > 0 || visitor.append_crosses_function(name) {
        visitor.mark_dynamic_keep_known_statement(name);
        return;
    }
    if visitor.control_depth > 0 {
        visitor.mark_dynamic_keep_known_statement(name);
        return;
    }
    let offset = visitor
        .lookup(name)
        .and_then(|binding| binding.sql)
        .map(|sql| count_placeholders(&sql))
        .unwrap_or(0);
    let (fragment_line, fragment_positions) = positions::fragment(call, visitor, offset);
    for scope in visitor.scopes.iter_mut().rev() {
        if let Some(binding) = scope.get_mut(name) {
            match (&binding.sql, binding.kind) {
                (Some(sql), EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed) => {
                    positions::append(
                        &mut binding.sql_source_positions,
                        sql,
                        binding.line,
                        fragment_line,
                        &fragment_positions,
                    );
                    let arg = renumber_placeholders(&arg, count_placeholders(sql));
                    binding.sql = Some(format!("{sql}{arg}"));
                    binding.kind = EmbeddedSqlKind::Composed;
                }
                _ => {
                    super::super::scope::mark_binding_dynamic_keep_known_statement(binding);
                }
            }
            return;
        }
    }
}

/// Appends an imported builder helper's text to `name`, as `name.append(text)`
/// would: straight-line code composes, anything conditional keeps only the
/// verified leading statement.
pub(crate) fn apply_hole_append(visitor: &mut ScopeVisitor<'_>, name: &str, hole: &RecoveredSql) {
    if visitor.loop_depth > 0 || visitor.append_crosses_function(name) || visitor.control_depth > 0
    {
        visitor.mark_dynamic_keep_known_statement(name);
        return;
    }
    for scope in visitor.scopes.iter_mut().rev() {
        if let Some(binding) = scope.get_mut(name) {
            match (&binding.sql, binding.kind) {
                (Some(sql), EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed) => {
                    positions::append(
                        &mut binding.sql_source_positions,
                        sql,
                        binding.line,
                        hole.line,
                        &hole.positions,
                    );
                    binding.sql = Some(format!("{sql}{}", hole.sql));
                    binding.kind = EmbeddedSqlKind::Composed;
                }
                _ => super::super::scope::mark_binding_dynamic_keep_known_statement(binding),
            }
            return;
        }
    }
}

fn first_static_arg(call: &CallExpression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let expression = call.arguments.first()?.as_expression()?;
    // `.append(fragment)` copies a tagged const's SQL. An untagged template
    // would stringify that object, so that path stays in `static_fragment`.
    if let Expression::Identifier(ident) = unwrap_ts_wrappers(expression) {
        return super::super::imported::constants::appended_identifier(visitor, ident);
    }
    if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(expression) {
        if let Some(hole) =
            super::super::imported::constants::namespace::piece(visitor, member, true)
        {
            return Some(hole);
        }
    }
    static_fragment(expression, visitor)
}
