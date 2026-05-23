fn assign_literal_cwd(cwd: &mut Option<PathBuf>, expr: &Expression) {
    *cwd = literal_string(expr).map(PathBuf::from);
}

// ── String extraction helpers ─────────────────────────────────────────────────

/// Extract a string literal or template literal (quasis concatenated) from an argument.
fn string_or_template_arg(args: &[Argument], index: usize) -> Option<String> {
    let arg = args.get(index)?;
    let expr = arg.as_expression()?;
    string_or_template_literal(expr)
}

/// Extract a string literal or template literal (quasis concatenated) from an expression.
fn string_or_template_literal(expr: &Expression) -> Option<String> {
    let expr = unwrap_ts_wrappers(expr);
    match expr {
        Expression::StringLiteral(s) => Some(s.value.as_str().to_string()),
        Expression::TemplateLiteral(tl) => {
            // Concatenate quasi strings (static parts), replacing interpolations with "".
            let parts: Vec<&str> = tl
                .quasis
                .iter()
                .filter_map(|q| q.value.cooked.as_deref())
                .collect();
            Some(parts.join(""))
        }
        _ => None,
    }
}

fn literal_string(expr: &Expression) -> Option<String> {
    let expr = unwrap_ts_wrappers(expr);
    if let Expression::StringLiteral(s) = expr {
        Some(s.value.as_str().to_string())
    } else {
        None
    }
}

/// Extract `cwd` from the opts object at `args[opts_index]`.
fn extract_cwd_from_opts(args: &[Argument], opts_index: usize) -> Option<String> {
    let obj = match args
        .get(opts_index)?
        .as_expression()
        .map(unwrap_ts_wrappers)
    {
        Some(Expression::ObjectExpression(obj)) => obj,
        _ => return None,
    };
    obj.properties
        .iter()
        .filter_map(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(p) => Some(p),
            _ => None,
        })
        .find_map(|p| {
            matches!(&p.key, PropertyKey::StaticIdentifier(id) if id.name.as_str() == "cwd")
                .then(|| literal_string(&p.value))
                .flatten()
        })
}

/// Attempt to get the short name of a callee (last identifier in a chain).
fn callee_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let expr = unwrap_ts_wrappers(expr);
    match expr {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(m) => Some(m.property.name.as_str()),
        _ => None,
    }
}

// ── Entry file resolution ─────────────────────────────────────────────────────
