/// Keep the standalone entry points of prepared SQL rules on one source/fact session.
macro_rules! prepared_rule_entrypoints {
    ($prepare:path $(, $precheck:ident)?) => {
        pub(crate) fn check_with_files(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            super::check_with_source_store(root, config, all_files, check_with_files_and_sources)
        }

        pub(crate) fn check_with_files_and_sources(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
            sources: &std::sync::Arc<crate::codebase::ts_source::SourceStore>,
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            $(let _ = $precheck(config)?;)?
            $prepare(root, config, all_files, sources, RULE_ID, check_with_files_sources_and_facts)
        }
    };
}
pub(crate) use prepared_rule_entrypoints;

/// The language rules share the same source-owned and deferred-suppression entry points.
macro_rules! borrowed_source_rule_entrypoints {
    () => {
        pub(crate) fn check_with_files(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            super::check_with_borrowed_source_store(
                root,
                config,
                all_files,
                check_with_files_and_sources,
            )
        }

        pub(crate) fn check_with_files_and_sources(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
            sources: &crate::codebase::ts_source::SourceStore,
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            check_with_files_sources_and_deferred_suppression(
                root, config, all_files, sources, false,
            )
        }
    };
}
pub(crate) use borrowed_source_rule_entrypoints;

/// File-scoped SQL rules with infallible option compilation share this fact projection.
macro_rules! prepared_file_rule_facts {
    () => {
        pub(crate) fn check_with_files_sources_and_facts(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
            sources: &crate::codebase::ts_source::SourceStore,
            facts: &crate::codebase::check_facts::CheckFactMap,
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            super::run_prepared_file_rule(
                super::PreparedRuleContext::new(root, config, all_files, sources, facts),
                RULE_ID,
                |rule| Ok(compile_options(&rule.try_rule_options()?)),
                scan::scan,
            )
        }
    };
}
pub(crate) use prepared_file_rule_facts;

/// Standalone source rules with parsed options and prepared inventory selection.
macro_rules! sourced_file_rule_entrypoints {
    () => {
        pub fn check(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            super::check_sourced_standalone(root, config, check_with_files)
        }

        pub(crate) fn check_with_files(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            super::check_with_borrowed_source_store(
                root,
                config,
                all_files,
                check_with_files_and_sources,
            )
        }

        pub(crate) fn check_with_files_and_sources(
            root: &std::path::Path,
            config: &crate::config::v2::NoMistakesConfig,
            all_files: &[std::path::PathBuf],
            sources: &crate::codebase::ts_source::SourceStore,
        ) -> anyhow::Result<Vec<super::RuleFinding>> {
            super::run_sourced_file_rule(root, config, all_files, sources, RULE_ID, scan)
        }
    };
}
pub(crate) use sourced_file_rule_entrypoints;
