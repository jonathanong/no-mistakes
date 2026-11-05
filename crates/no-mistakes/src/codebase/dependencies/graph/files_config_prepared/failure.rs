#[doc(hidden)]
pub struct PlaywrightPreparationFailure {
    pub file: Option<PathBuf>,
    pub project: Option<String>,
    pub config_name: Option<String>,
    pub message: String,
    pub(crate) roots: Vec<PathBuf>,
}

impl PlaywrightPreparationFailure {
    pub(crate) fn new(
        root: &Path,
        settings: &crate::playwright::config::Settings,
        file: Option<PathBuf>,
        project: Option<String>,
        config_name: Option<String>,
        error: anyhow::Error,
    ) -> Self {
        let roots = std::iter::once(&settings.frontend_root)
            .chain(&settings.selector_roots)
            .map(|path| crate::codebase::ts_resolver::normalize_path(&root.join(path)))
            .collect();
        Self {
            file,
            project,
            config_name,
            message: format!("{error:#}"),
            roots,
        }
    }

    #[doc(hidden)]
    pub fn affects_rule(
        &self,
        root: &Path,
        config: &crate::config::v2::NoMistakesConfig,
        rule: &crate::config::v2::schema::RuleDef,
        files: &[PathBuf],
    ) -> anyhow::Result<bool> {
        if !rule.tests.playwright.is_empty()
            && (self.project.is_some() || self.config_name.is_some())
            && !rule.tests.playwright.iter().any(|selected| {
                self.project.as_ref() == Some(selected)
                    || self.config_name.as_ref() == Some(selected)
            })
        {
            return Ok(false);
        }
        let binding = self
            .project
            .as_ref()
            .or(self.config_name.as_ref())
            .and_then(|project| config.tests.playwright.apps.get(project));
        if !rule.projects.is_empty()
            && binding
                .and_then(|binding| binding.project.as_ref())
                .is_some_and(|project| !rule.projects.contains(project))
        {
            return Ok(false);
        }
        let filter = crate::codebase::rules::path_filter::RulePathFilter::new(root, config, rule)?;
        Ok(files.iter().any(|file| {
            self.roots.iter().any(|scope| file.starts_with(scope)) && filter.is_match(file)
        }))
    }
}

#[cfg(test)]
#[path = "failure_tests.rs"]
mod playwright_failure_tests;
