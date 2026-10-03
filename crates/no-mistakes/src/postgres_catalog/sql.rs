use super::PostgresCatalogCoverage;

/// A double-quoted PostgreSQL identifier, safe for any schema name.
pub(super) fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// An `E''` string literal that is correct whatever `standard_conforming_strings` is.
fn quote_literal(value: &str) -> String {
    format!("E'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
}

/// The whole read-only script for one schema and coverage.
///
/// `pg_catalog` leads `search_path`, so objects in the selected schema cannot shadow the built-ins
/// this query calls. The selected schema follows, so its types render unqualified, the way the
/// rules compare them, while types from other schemas stay qualified.
pub(super) fn catalog_query(schema: &str, coverage: PostgresCatalogCoverage) -> String {
    let complete = coverage == PostgresCatalogCoverage::Complete;
    // The schema is substituted last: its text must never be scanned for placeholders.
    let query = include_str!("catalog.sql")
        .replace(
            "__COVERAGE__",
            if complete { "'complete'" } else { "'ordering'" },
        )
        .replace("__COMPLETE__", if complete { "true" } else { "false" })
        .replace("__SCHEMA__", &quote_literal(schema));
    format!(
        "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;\n\
         SET LOCAL standard_conforming_strings = on;\n\
         SET LOCAL statement_timeout = '30s';\n\
         SET LOCAL client_encoding = 'UTF8';\n\
         SET LOCAL search_path = pg_catalog, {};\n\
         {query}COMMIT;\n",
        quote_identifier(schema)
    )
}
