use super::RULE_ID;
use crate::codebase::rules::path_filter::GlobMatcher;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::schema::RuleDef;
use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct Options {
    production_roots: Vec<String>,
    test_globs: Vec<String>,
    test_helper_roots: Vec<String>,
}

pub(super) struct Selection {
    root: PathBuf,
    production_roots: Vec<PathBuf>,
    tests: GlobMatcher,
    helpers: Vec<PathBuf>,
}

impl Selection {
    pub(super) fn new(root: &Path, application: &RuleDef) -> Result<Self> {
        let options: Options = application.try_rule_options()?;
        if options.production_roots.is_empty() {
            bail!("{RULE_ID}: options.productionRoots must contain at least one configured production root");
        }
        if options.test_globs.is_empty() && options.test_helper_roots.is_empty() {
            bail!("{RULE_ID}: configure options.testGlobs or options.testHelperRoots to classify test consumers");
        }
        if options
            .test_globs
            .iter()
            .any(|value| value.trim().is_empty())
        {
            bail!("{RULE_ID}: options.testGlobs must not contain empty strings");
        }
        for value in options
            .production_roots
            .iter()
            .chain(&options.test_helper_roots)
        {
            if value.trim().is_empty() {
                bail!("{RULE_ID}: configured roots must not be empty strings");
            }
        }
        Ok(Self {
            root: root.to_path_buf(),
            production_roots: roots(root, options.production_roots),
            tests: GlobMatcher::new(&options.test_globs, &format!("{RULE_ID} options.testGlobs"))?,
            helpers: roots(root, options.test_helper_roots),
        })
    }

    pub(super) fn production(&self, file: &Path) -> bool {
        self.production_roots
            .iter()
            .any(|root| file.starts_with(root))
            && !self.test(file)
    }

    pub(super) fn test(&self, file: &Path) -> bool {
        self.helpers.iter().any(|root| file.starts_with(root))
            || self.tests.is_match(&relative_slash_path(&self.root, file))
    }
}

fn roots(root: &Path, values: Vec<String>) -> Vec<PathBuf> {
    values
        .into_iter()
        .map(|value| normalize_path(&root.join(value)))
        .collect()
}
