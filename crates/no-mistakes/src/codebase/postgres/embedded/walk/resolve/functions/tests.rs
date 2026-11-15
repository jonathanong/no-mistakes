use super::*;

impl LocalFunctions {
    pub(crate) fn collect(program: &Program<'_>, source: &str, trusted: &[TrustedSqlTag]) -> Self {
        collection::collect(program, source, trusted, &[])
    }
}
