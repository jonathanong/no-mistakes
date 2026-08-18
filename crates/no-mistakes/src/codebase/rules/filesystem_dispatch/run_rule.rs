use super::*;

pub(super) struct RunRuleRequest<'a> {
    pub(super) rule_id: &'a str,
    pub(super) fallback:
        fn(&Path, &crate::config::v2::NoMistakesConfig, &[PathBuf]) -> Result<Vec<RuleFinding>>,
    pub(super) root: &'a Path,
    pub(super) config: &'a crate::config::v2::NoMistakesConfig,
    pub(super) files: &'a [PathBuf],
    pub(super) sources: &'a crate::codebase::ts_source::SourceStore,
    pub(super) facts: Option<&'a crate::codebase::check_facts::CheckFactMap>,
    pub(super) defer_suppression: bool,
}

pub(super) fn run_rule_with_sources(request: RunRuleRequest<'_>) -> Result<Vec<RuleFinding>> {
    let RunRuleRequest {
        rule_id,
        fallback,
        root,
        config,
        files,
        sources,
        facts,
        defer_suppression,
    } = request;
    match rule_id {
        AGENTS_MD_MAX_SIZE => {
            agents_md_max_size::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        FINITE_SET_CONSISTENCY => finite_set_consistency::check_with_files_sources_and_facts(
            root,
            config,
            files,
            sources,
            facts.map(|facts| facts as &dyn crate::codebase::dependencies::graph::TsFactLookup),
        ),
        FORBIDDEN_WORKSPACE_CLOSURE => {
            forbidden_workspace_closure::check_with_files_and_sources(root, config, files, sources)
        }
        REQUIRED_COMPANION_IMPORTS => {
            required_companion_imports::check_with_files_and_sources(root, config, files, sources)
        }
        REQUIRED_DOC_SECTION => {
            required_local_docs::check_required_doc_section_with_files_and_sources(
                root, config, files, sources,
            )
        }
        DOC_CONSISTENCY => {
            doc_consistency::check_with_files_and_sources(root, config, files, sources)
        }
        SHELLCHECK_RUNNER => {
            shellcheck_runner::check_with_files_and_sources(root, config, files, sources)
        }
        PACKAGE_JSON_WORKSPACE_COVERAGE => {
            package_json_workspace_coverage::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        WORKSPACE_PACKAGE_CYCLES => {
            workspace_package_cycles::check_with_files_and_sources(root, config, files, sources)
        }
        PACKAGE_JSON_REGISTRY_ONLY => {
            package_json_registry_only::check_with_files_and_sources(root, config, files, sources)
        }
        PRODUCTION_DEPENDENCY_DECLARATIONS => {
            production_dependency_declarations::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        NO_GIT_IDENTITY_MUTATION => {
            no_git_identity_mutation::check_with_files_and_sources(root, config, files, sources)
        }
        TEST_EMAIL_DOMAIN_POLICY => {
            test_email_domain_policy::check_with_files_and_sources(root, config, files, sources)
        }
        INTEGRATION_TEST_NO_MOCKS => {
            integration_test_no_mocks::check_with_files_and_sources(root, config, files, sources)
        }
        MARKDOWN_LINK_DISPLAY_TEXT => {
            markdown_link_display_text::check_with_files_and_sources(root, config, files, sources)
        }
        STRUCTURED_CONFIG_POLICY => {
            structured_config_policy::check_with_files_and_sources(root, config, files, sources)
        }
        CONFIG_PATH_REFERENCES => {
            config_path_references::check_with_files_and_sources(root, config, files, sources)
        }
        CSHARP_MAX_LINES_PER_FILE => {
            csharp_max_lines_per_file::check_with_files_sources_and_deferred_suppression(
                root,
                config,
                files,
                sources,
                defer_suppression,
            )
        }
        GITHUB_ACTIONS_PINNED_HASH => {
            github_actions_pinned_hash::check_with_files_and_sources(root, config, files, sources)
        }
        NO_EMPTY_OR_COMMENTS_ONLY_FILES => {
            no_empty_or_comments_only_files::check_with_files_and_sources(
                root, config, files, sources,
            )
        }
        _ => fallback(root, config, files),
    }
}
