use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::codebase::rules) struct Options {
    /// Repository-relative globs for files that only exercise code, such as a
    /// shared `test-helpers/` directory. They extend the built-in `__tests__`
    /// and `.test.`/`.spec.` classification.
    pub(super) test_files: Vec<String>,
}

impl Options {
    pub(in crate::codebase::rules) fn test_file_matcher(
        &self,
    ) -> anyhow::Result<crate::codebase::rules::path_filter::GlobMatcher> {
        crate::codebase::rules::path_filter::GlobMatcher::new(
            &self.test_files,
            "unconstructed-error-class options.testFiles",
        )
    }
}
