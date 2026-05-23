pub fn unwrap_ts_wrappers<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::TSAsExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSNonNullExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSTypeAssertion(e) => unwrap_ts_wrappers(&e.expression),
        Expression::TSSatisfiesExpression(e) => unwrap_ts_wrappers(&e.expression),
        Expression::ParenthesizedExpression(e) => unwrap_ts_wrappers(&e.expression),
        other => other,
    }
}

pub fn static_property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
        _ => None,
    }
}

/// Returns `true` if the first non-empty line of `source` is a `'use client'` or
/// `"use client"` directive prologue. Checks only the first line to avoid false positives
/// from occurrences inside comments or string literals later in the file.
pub fn starts_with_use_client(source: &str) -> bool {
    let first_line = source
        .trim_start_matches('\u{FEFF}') // strip optional BOM
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

