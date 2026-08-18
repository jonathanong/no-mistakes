//! Cheap DML write-target extraction and generated-column write matching.

use regex::Regex;
use std::sync::LazyLock;

mod writes;

pub use writes::{
    find_generated_column_writes, GeneratedColumnWrite, GeneratedTable, GeneratedTableColumns,
};

/// Matches `UPDATE`, `INSERT INTO`, and `MERGE INTO` table names.
static WRITE_TARGET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?is)\b(?:UPDATE|INSERT\s+INTO|MERGE\s+INTO)\s+(?:ONLY\s+)?((?:"[^"]+"|[A-Za-z_][\w$]*)(?:\s*\.\s*(?:"[^"]+"|[A-Za-z_][\w$]*))*)"#,
    )
    .expect("dml write-target regex")
});

static IDENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:"([^"]+)"|([A-Za-z_][\w$]*))"#).expect("dml identifier regex")
});

/// Table names targeted by UPDATE / INSERT INTO / MERGE INTO.
///
/// This is a cheap prefilter. It does not parse SQL. `UPDATE SET` (no table)
/// can yield a dummy `SET` token, which later parse matching ignores.
pub fn extract_dml_write_targets(sql: &str) -> Vec<String> {
    let mut targets: Vec<String> = WRITE_TARGET
        .captures_iter(sql)
        .filter_map(|caps| {
            caps.get(1)
                .map(|matched| last_relation_name(matched.as_str()))
        })
        .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("SET"))
        .collect();
    targets.sort();
    targets.dedup();
    targets
}

fn last_relation_name(qualified: &str) -> String {
    IDENT
        .captures_iter(qualified)
        .filter_map(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .map(|part| part.as_str().to_string())
        })
        .last()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
