use anyhow::Result;

/// Parse `unanalyzableSql`: empty/`fail` fail closed, `ignore` does not.
pub fn fail_unanalyzable_sql(rule: &str, value: &str) -> Result<bool> {
    match value.trim() {
        "" => Ok(true),
        other if other.eq_ignore_ascii_case("fail") => Ok(true),
        other if other.eq_ignore_ascii_case("ignore") => Ok(false),
        other => anyhow::bail!("{rule}: unanalyzableSql must be `fail` or `ignore`, got `{other}`"),
    }
}

#[cfg(test)]
mod tests;
