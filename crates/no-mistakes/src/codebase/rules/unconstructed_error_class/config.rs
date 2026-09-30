use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Options {
    /// Repository-relative globs for files that only exercise code, such as a
    /// shared `test-helpers/` directory. They extend the built-in `__tests__`
    /// and `.test.`/`.spec.` classification.
    pub(super) test_files: Vec<String>,
}
