use super::super::ScopeVisitor;
use super::{expression, Recovered, ValueKind};
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

/// The outer Option distinguishes ordinary calls from a known parameter
/// helper that must fail closed instead of falling back to call-span origins.
pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Option<Vec<Recovered>>> {
    let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let builder = visitor.functions.parameter_builder(name.name.as_str())?;
    Some((|| {
        if visitor.shadowed_locally(name.name.as_str()) {
            return None;
        }
        let argument = builder.checked_argument(call)?;
        match unwrap_ts_wrappers(argument) {
            Expression::Identifier(name) => {
                let binding = visitor.lookup(name.name.as_str())?;
                if !binding.sql_builder || binding.kind == EmbeddedSqlKind::Dynamic {
                    return None;
                }
            }
            Expression::TaggedTemplateExpression(_) => {}
            _ => return None,
        }
        let mut versions = expression::recover(visitor, argument, depth, constraints)?;
        if !versions
            .iter()
            .all(|value| value.value == ValueKind::Sql && value.fragment)
        {
            return None;
        }
        for fragment in &builder.fragments {
            let suffix = Recovered {
                value: ValueKind::Sql,
                sql: fragment.sql.clone(),
                line: fragment.line,
                origins: fragment.origins.clone(),
                positions: fragment.positions.clone(),
                fragment: true,
                enumerated: false,
                choices: Vec::new(),
            };
            // A prepared suffix has no branch choices, so appending it to
            // each already-capped SQL version cannot increase the count.
            for version in &mut versions {
                version.append_sql(&suffix);
            }
        }
        Some(versions)
    })())
}
