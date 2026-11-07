use super::*;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Options {
    pub(crate) config: String,
    pub(crate) helpers: Vec<String>,
    pub(crate) required: BTreeMap<String, serde_json::Value>,
    pub(crate) forbidden: Vec<String>,
    pub(crate) parity: Vec<Parity>,
    pub(crate) declared: Option<Declared>,
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
    pub(super) fn check_effective(&self) -> bool {
        self.declared.is_none()
            || !self.required.is_empty()
            || !self.forbidden.is_empty()
            || !self.parity.is_empty()
    }

    pub(super) fn source_paths(
        &self,
        root: &Path,
        config: &Path,
        files: &[PathBuf],
    ) -> Result<BTreeSet<PathBuf>> {
        let mut paths =
            super::super::file_matching::matching_files(root, &self.helpers, files, &[])?
                .into_iter()
                .collect::<BTreeSet<_>>();
        paths.insert(config.to_path_buf());
        paths.extend(
            self.helpers
                .iter()
                .filter(|helper| !helper.contains(['*', '?', '[', '{']))
                .map(|helper| crate::codebase::ts_resolver::normalize_path(&root.join(helper))),
        );
        Ok(paths)
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
