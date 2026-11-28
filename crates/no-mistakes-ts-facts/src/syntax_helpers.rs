use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, PropertyKey, TemplateLiteral};
use oxc_diagnostics::OxcDiagnostic;
use std::path::Path;

pub fn unwrap_ts_wrappers<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::TSAsExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSNonNullExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSTypeAssertion(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSSatisfiesExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSInstantiationExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::ParenthesizedExpression(e) => unwrap_ts_wrappers(&e.expression),
        other => other,
    }
}

pub fn static_property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<std::borrow::Cow<'a, str>> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(std::borrow::Cow::Borrowed(id.name.as_str())),
        PropertyKey::StringLiteral(s) => Some(crate::ast::js_literal(s)),
        _ => None,
    }
}

/// Returns `true` if the first non-empty line of `source` is a `'use client'` or
/// `"use client"` directive prologue. Checks only the first line to avoid false positives
/// from occurrences inside comments or string literals later in the file.
pub fn starts_with_use_client(source: &str) -> bool {
    let first_line = source
        .trim_start_matches('\u{FEFF}')
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    matches!(
        first_line.trim(),
        "'use client'" | "'use client';" | "\"use client\"" | "\"use client\";"
    )
}

/// Returns `true` if `relative` is a test file — either living under an
/// `/__tests__/` directory or having a `.test.*` / `.spec.*` suffix that matches
/// the pattern `\.(test|spec)\.[cm]?[jt]sx?`.
pub fn is_test_file(relative: &str) -> bool {
    if relative.contains("/__tests__/") {
        return true;
    }
    const SUFFIXES: &[&str] = &[
        ".test.ts",
        ".test.tsx",
        ".test.js",
        ".test.jsx",
        ".test.mts",
        ".test.cts",
        ".test.mjs",
        ".test.cjs",
        ".spec.ts",
        ".spec.tsx",
        ".spec.js",
        ".spec.jsx",
        ".spec.mts",
        ".spec.cts",
        ".spec.mjs",
        ".spec.cjs",
    ];
    SUFFIXES.iter().any(|s| relative.ends_with(s))
}

/// Format the first parser diagnostic, retaining the panic fallback used by
/// callers when the parser aborts without providing diagnostic details.
pub fn format_parse_diagnostic(path: &Path, diagnostics: &[OxcDiagnostic]) -> String {
    match diagnostics.first() {
        Some(error) => format!("parsing {}: {error:?}", path.display()),
        None => format!(
            "parsing {}: parser panicked without diagnostic details",
            path.display()
        ),
    }
}

pub fn normalize_next_pathname_pattern(path: &str) -> String {
    let leading_slash = path.starts_with('/');
    let trailing_slash = path.ends_with('/') && path.len() > 1;
    let segments: Vec<String> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            if segment.starts_with("[[...") && segment.ends_with("]]") {
                "**".to_string()
            } else if segment.starts_with("[...") && segment.ends_with(']') {
                "*".to_string()
            } else if segment.starts_with('[') && segment.ends_with(']') {
                format!(":{}", &segment[1..segment.len() - 1])
            } else {
                segment.to_string()
            }
        })
        .collect();

    let mut normalized = if leading_slash {
        format!("/{}", segments.join("/"))
    } else {
        segments.join("/")
    };
    if trailing_slash {
        normalized.push('/');
    }
    normalized
}

/// Normalize a template literal to a route pattern (replaces `${...}` with `:param`).
pub fn normalize_template(tpl: &TemplateLiteral) -> String {
    let mut result = String::new();
    for (i, quasi) in tpl.quasis.iter().enumerate() {
        if quasi.value.cooked.is_some() {
            result.push_str(crate::ast::template_quasi_text(quasi).as_ref());
        }
        if i < tpl.expressions.len() {
            result.push_str(":param");
        }
    }
    normalize_next_pathname_pattern(&result)
}

/// Extract `cwd` from the opts object at `args[opts_index]`.
pub fn extract_cwd_from_opts(args: &[Argument], opts_index: usize) -> Option<String> {
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
                .then(|| match unwrap_ts_wrappers(&p.value) {
                    Expression::StringLiteral(s) => Some(crate::ast::js_literal(s).into_owned()),
                    _ => None,
                })
                .flatten()
        })
}

/// Convert a byte offset in `source` into a 1-based line number.
pub fn byte_offset_to_line(source: &str, offset: usize) -> u32 {
    let clamped = offset.min(source.len());
    (source.as_bytes()[..clamped]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1) as u32
}
