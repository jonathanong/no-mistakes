#[cfg(any(test, feature = "test-instrumentation"))]
pub(super) mod key_work;

pub(crate) use no_mistakes_postgres_sql::sql_names::{
    decoded_parts, normalize_identifier, normalize_table_name, parts, plain_column,
};

/// A SQL name split into its schema qualifier, if it has one, and its bare name, both decoded
/// the way PostgreSQL reads them (an unquoted part folds to lower case):
/// `public."audit.log"` is `(Some("public"), "audit.log")`.
pub(super) fn split_name(name: &str) -> (Option<String>, String) {
    split(decoded_parts(name))
}

/// The same for a key of the catalog, which is already normalized: its parts are unquoted but
/// never case-folded again, since a mixed-case key (`Order Items`) is exact.
pub(super) fn split_key(key: &str) -> (Option<String>, String) {
    #[cfg(any(test, feature = "test-instrumentation"))]
    key_work::record();
    let unquote = |part: &String| match part.strip_prefix('"').and_then(|p| p.strip_suffix('"')) {
        Some(inner) => inner.replace("\"\"", "\""),
        None => part.clone(),
    };
    split(parts(key).iter().map(unquote).collect())
}

fn split(mut parts: Vec<String>) -> (Option<String>, String) {
    let bare = parts.pop().unwrap_or_default();
    ((!parts.is_empty()).then(|| parts.join(".")), bare)
}
