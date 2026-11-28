#[doc(hidden)]
pub fn assign_literal_cwd(cwd: &mut Option<PathBuf>, expr: &Expression) {
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
        Expression::StringLiteral(s) => Some(no_mistakes_ts_facts::ast::js_literal(s).into_owned()),
        Expression::TemplateLiteral(tl) => {
            // Concatenate quasi strings (static parts), replacing interpolations with "".
            let parts: Vec<_> = tl
                .quasis
                .iter()
                .filter_map(|q| q.value.cooked.map(|_| no_mistakes_ts_facts::ast::template_quasi_text(q)))
                .collect();
            Some(parts.join(""))
        }
        _ => None,
    }
}

fn literal_string(expr: &Expression) -> Option<String> {
    let expr = unwrap_ts_wrappers(expr);
    if let Expression::StringLiteral(s) = expr {
        Some(no_mistakes_ts_facts::ast::js_literal(s).into_owned())
    } else {
        None
    }
}

pub use no_mistakes_ts_facts::syntax_helpers::extract_cwd_from_opts;

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
