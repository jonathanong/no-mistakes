use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SubstringSpec {
    pub(crate) file: String,
    pub(crate) substring: String,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct BannedPattern {
    pub(crate) files: Vec<String>,
    pub(crate) pattern: String,
    pub(crate) message: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) required_files: Vec<String>,
    pub(crate) required_heading: Option<String>,
    pub(crate) required_substrings: Vec<SubstringSpec>,
    pub(crate) banned_substrings: Vec<String>,
    pub(crate) banned_patterns: Vec<BannedPattern>,
}

impl Options {
    pub(super) fn retain_selected_substrings(
        &mut self,
        root: &std::path::Path,
        config: &crate::config::v2::NoMistakesConfig,
        target_roots: &[std::path::PathBuf],
        filter: &super::super::path_filter::RulePathFilter,
    ) {
        let skip = super::super::skip_dir_set(config);
        self.required_substrings.retain(|spec| {
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(&spec.file));
            filter.is_match(&path)
                && super::super::file_allowed_by_roots_and_skip(root, &skip, &path, target_roots)
        });
    }
}
