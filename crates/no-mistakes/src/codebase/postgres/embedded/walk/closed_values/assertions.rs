use super::*;

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    depth: u8,
) -> Option<Vec<Arm>> {
    if call.optional
        || call
            .arguments
            .iter()
            .any(|arg| matches!(arg, Argument::SpreadElement(_)))
    {
        return None;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let name = id.name.as_str();
    if visitor.shadowed_locally(name) {
        return None;
    }
    if let Some(assertion) = visitor.literal_objects.closed.assertions.get(name) {
        return assertion_values(visitor, call, assertion.allowlist_argument, depth);
    }
    helper_values(visitor, call, name, depth)
}
fn assertion_values(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    index: usize,
    depth: u8,
) -> Option<Vec<Arm>> {
    let argument = call.arguments.get(index)?.as_expression()?;
    let argument = unwrap_ts_wrappers(argument);
    let values = if let Expression::NewExpression(created) = argument {
        let Expression::Identifier(constructor) = unwrap_ts_wrappers(&created.callee) else {
            return None;
        };
        if constructor.name != "Set"
            || !visitor.set_intrinsic_safe
            || visitor.lookup("Set").is_some()
            || created.arguments.len() != 1
        {
            return None;
        }
        expression(visitor, created.arguments[0].as_expression()?, depth - 1)?
    } else {
        expression(visitor, argument, depth - 1)?
    };
    let mut result = Vec::new();
    for arm in values {
        let Node::Array(items) = arm.node else {
            return None;
        };
        for (i, item) in items.into_iter().enumerate() {
            let Node::Text(_) = item else { return None };
            let mut choices = arm.choices.clone();
            choices.push((call.span.start as u64 | (1 << 63), i as u32));
            result.push(Arm {
                node: item,
                choices,
                root: arm.root.clone(),
                provenance: None,
                type_annotation: false,
            });
        }
    }
    (!result.is_empty() && result.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(result)
}
fn helper_values(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    name: &str,
    depth: u8,
) -> Option<Vec<Arm>> {
    if !visitor.functions.has_program(name) {
        return None;
    }
    let args = super::super::imported::clause_args(visitor, call)?;
    let mut alternatives = vec![args];
    for (index, argument) in call.arguments.iter().enumerate() {
        let argument = argument.as_expression()?;
        let Some(values) = expression(visitor, argument, depth - 1) else {
            continue;
        };
        let mut expanded = Vec::new();
        for args in alternatives {
            for value in &values {
                if value.root.is_some() {
                    return None;
                }
                let Node::Text(text) = &value.node else {
                    return None;
                };
                let mut args = args.clone();
                args[index] = crate::codebase::postgres::embedded::helpers::SiteArg::Literal(
                    crate::codebase::postgres::embedded::helpers::LiteralArg {
                        text: text.clone(),
                        line: byte_offset_to_line(visitor.source, argument.span().start as usize),
                        origin: argument.span().start,
                    },
                );
                expanded.push(args);
            }
        }
        if expanded.len() > MAX_EMBEDDED_SQL_VARIANTS {
            return None;
        }
        alternatives = expanded;
    }
    let mut strings = Vec::new();
    for args in alternatives {
        strings.extend(visitor.functions.identifier_clause_values(name, &args)?);
    }
    Some(
        strings
            .into_iter()
            .enumerate()
            .map(|(i, (text, provenance))| Arm {
                node: Node::Text(text),
                choices: vec![(call.span.start as u64 | (1 << 63), i as u32)],
                root: None,
                provenance,
                type_annotation: false,
            })
            .collect(),
    )
}
