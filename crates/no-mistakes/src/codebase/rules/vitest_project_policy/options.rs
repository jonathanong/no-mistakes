use super::*;
use anyhow::Context;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) config: String,
    pub(crate) helpers: Vec<String>,
    pub(crate) required: BTreeMap<String, serde_json::Value>,
    pub(crate) forbidden: Vec<String>,
    pub(crate) required_exclude_globs: Vec<String>,
    pub(crate) parity: Vec<Parity>,
    pub(super) declared: Option<Declared>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Declared {
    pub(super) forbidden_values: BTreeMap<String, Vec<serde_json::Value>>,
    pub(super) forbid_literal_kinds: BTreeMap<String, Vec<LiteralKind>>,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum LiteralKind {
    Number,
    String,
    Boolean,
}

impl Options {
    pub(super) fn checks_exclude_globs(&self) -> bool {
        !self.required_exclude_globs.is_empty()
    }

    pub(super) fn enforces_project_literals(&self) -> bool {
        let project_policy =
            !self.required.is_empty() || !self.forbidden.is_empty() || !self.parity.is_empty();
        if self.declared.is_some() && !project_policy {
            return false;
        }
        if self.checks_exclude_globs() && self.declared.is_none() && !project_policy {
            return false;
        }
        true
    }

    pub(super) fn check_effective(&self) -> bool {
        self.enforces_project_literals() || self.checks_exclude_globs()
    }

    pub(super) fn source_paths(
        &self,
        root: &Path,
        config: &Path,
        files: &[PathBuf],
    ) -> Result<BTreeSet<PathBuf>> {
        let mut paths = BTreeSet::from([config.to_path_buf()]);
        let mut patterns = Vec::new();
        for helper in &self.helpers {
            let path = crate::codebase::ts_resolver::normalize_path(&root.join(helper));
            if path.exists() || !helper.contains(['*', '?', '[', '{']) {
                paths.insert(path);
            } else {
                patterns.push(helper.clone());
            }
        }
        paths.extend(
            super::super::file_matching::matching_files(root, &patterns, files, &[])
                .context("invalid options.helpers")?,
        );
        Ok(paths)
    }

    pub(in crate::codebase::rules) fn validate_for_config_load(&self, root: &Path) -> Result<()> {
        self.validate()?;
        self.source_paths(root, &root.join(&self.config), &[])?;
        Ok(())
    }

    pub(super) fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            !self.config.is_empty(),
            "{RULE_ID} requires options.config naming an explicit config file"
        );
        anyhow::ensure!(
            self.required.values().all(scalar),
            "{RULE_ID} required values must be scalar JSON literals"
        );
        anyhow::ensure!(
            self.required_exclude_globs
                .iter()
                .all(|glob| !glob.is_empty()),
            "{RULE_ID} requiredExcludeGlobs entries must be nonempty strings"
        );
        let declared = self.declared.as_ref();
        anyhow::ensure!(
            declared
                .into_iter()
                .flat_map(|declared| declared.forbidden_values.values())
                .flatten()
                .all(scalar),
            "{RULE_ID} declared.forbiddenValues must contain scalar JSON literals"
        );
        anyhow::ensure!(
            self.required
                .keys()
                .chain(self.forbidden.iter())
                .chain(self.parity.iter().flat_map(|pair| pair.properties.iter()))
                .chain(declared.into_iter().flat_map(|declared| {
                    declared
                        .forbidden_values
                        .keys()
                        .chain(declared.forbid_literal_kinds.keys())
                }))
                .all(|key| !key.is_empty() && !key.contains('.')),
            "{RULE_ID} property names must be nonempty direct test property names"
        );
        Ok(())
    }
}

fn scalar(value: &serde_json::Value) -> bool {
    !value.is_array() && !value.is_object()
}
