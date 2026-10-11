use super::*;

pub(super) fn alias_or_append(
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

pub(super) fn append(
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
        super::super::super::for_each_bound_name(&param.pattern, &mut |name| {
            names.insert(name.to_string());
        });
    }
    if let Some(rest) = &params.rest {
        super::super::super::for_each_bound_name(&rest.rest.argument, &mut |name| {
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
        super::super::MAX_RESOLVE_DEPTH,
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
