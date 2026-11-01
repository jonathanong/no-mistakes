use super::*;

pub(in crate::codebase::postgres::embedded::walk) fn contains_parameter_helper(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> bool {
    let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
        return false;
    };
    match unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(ident) => {
            !visitor.shadowed_locally(ident.name.as_str())
                && (visitor.functions.is_parameter_builder(ident.name.as_str())
                    || visitor.is_hole_helper(ident.name.as_str())
                        && crate::codebase::postgres::embedded::walk::imported::passes_builder(
                            visitor, call,
                        ))
        }
        Expression::StaticMemberExpression(member) if member.property.name == "append" => {
            contains_parameter_helper(&member.object, visitor)
        }
        _ => false,
    }
}

pub(in crate::codebase::postgres::embedded::walk::resolve) fn parameter_source_positions(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<Vec<crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition>> {
    fn resolve(
        expr: &Expression<'_>,
        visitor: &ScopeVisitor<'_>,
    ) -> Option<(
        String,
        Vec<crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition>,
        u32,
    )> {
        let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) else {
            return None;
        };
        match unwrap_ts_wrappers(&call.callee) {
            Expression::Identifier(ident) if !visitor.shadowed_locally(ident.name.as_str()) => {
                visitor
                    .functions
                    .call_positions(call, ident.name.as_str(), visitor)
            }
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                let (base, mut positions, origin) = resolve(&member.object, visitor)?;
                let fragment = call.arguments.first()?.as_expression()?;
                let text = static_fragment(fragment, visitor)?;
                let (line, fragment_positions) = super::super::append::positions::fragment(
                    call,
                    visitor,
                    count_placeholders(&base),
                );
                super::super::append::positions::append(
                    &mut positions,
                    &base,
                    origin,
                    line,
                    &fragment_positions,
                );
                let text = renumber_placeholders(&text, count_placeholders(&base));
                Some((format!("{base}{text}"), positions, origin))
            }
            _ => None,
        }
    }
    resolve(expr, visitor).map(|(_, positions, _)| positions)
}

/// `head + tail` names consts. Their positions live on the bindings; the `+`
/// expression itself has no template span to measure.
pub(in crate::codebase::postgres::embedded::walk::resolve) fn binding_source_positions(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
    sql: &str,
) -> Option<Vec<crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition>> {
    if !mentions_binding(expr) {
        return None;
    }
    let recovered = super::resolve_chain_recovered(expr, visitor)?;
    if recovered.sql != sql {
        return None;
    }
    let mut positions = recovered.positions;
    // A fragment with no recorded position is still anchored at its declaration.
    let anchored = positions
        .first()
        .is_some_and(|position| position.sql_line == 1 && position.sql_column == 1);
    if !anchored {
        positions.insert(
            0,
            crate::codebase::postgres::embedded::EmbeddedSqlSourcePosition {
                sql_line: 1,
                sql_column: 1,
                source_line: recovered.line,
                source_path: None,
            },
        );
    }
    Some(positions)
}

fn mentions_binding(expr: &Expression<'_>) -> bool {
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(_) => true,
        Expression::BinaryExpression(binary) if binary.operator == BinaryOperator::Addition => {
            mentions_binding(&binary.left) || mentions_binding(&binary.right)
        }
        _ => false,
    }
}
