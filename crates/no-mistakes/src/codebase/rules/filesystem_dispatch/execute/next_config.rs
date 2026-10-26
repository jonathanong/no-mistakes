use super::*;

pub(super) fn run(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    prepared: PreparedFilesystemRuleInputs<'_>,
    facts: Option<&crate::codebase::check_facts::CheckFactMap>,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    if prepared.next_config_resolver.is_none() && rule_enabled(config, NEXTJS_REDIRECT_DESTINATIONS)
    {
        let sources = std::sync::Arc::clone(&prepared.sources);
        return nextjs_redirect_destinations::with_standalone_resolver(
            root,
            config,
            files,
            &sources,
            |resolver| {
                run_prepared_filesystem_rules(
                    root,
                    config,
                    files,
                    PreparedFilesystemRuleInputs {
                        next_config_resolver: Some(resolver),
                        ..prepared
                    },
                    facts,
                    defer_suppression,
                )
            },
        );
    }
    run_prepared_filesystem_rules(root, config, files, prepared, facts, defer_suppression)
}
