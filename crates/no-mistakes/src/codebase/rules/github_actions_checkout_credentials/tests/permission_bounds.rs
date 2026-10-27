use super::*;
use crate::codebase::rules::{analysis_policy, source_store_for_files};
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};

fn saved_config(root: &Path, name: &str) -> NoMistakesConfig {
    serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap()
}

#[test]
fn saved_bounds_stay_definite_across_policies_efforts_and_suppressions() {
    // Output cases moved to cli_gha_snapshot_checkout_credentials.
    let root = super::tests::fixture_root("permission-bounds");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    for effort in [Effort::Max, Effort::Low] {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            let suffix = match policy {
                Unanalyzable::Report => "report",
                Unanalyzable::Advise => "advise",
                Unanalyzable::Ignore => "ignore",
            };
            let name = match effort {
                Effort::Max if policy == Unanalyzable::Report => ".no-mistakes.yml".to_string(),
                Effort::Max => format!(".no-mistakes.max-{suffix}.yml"),
                Effort::Low => format!(".no-mistakes.low-{suffix}.yml"),
            };
            let mut config = saved_config(&root, &name);
            let sources = source_store_for_files(&files);
            analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
            assert_eq!(
                config.analysis_warnings.len(),
                usize::from(effort == Effort::Low)
            );
            let output =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                    .unwrap();
            for finding in output.findings.iter().chain(&output.advisories) {
                let text = std::fs::read_to_string(root.join(&finding.file)).unwrap();
                assert!(
                    text.lines()
                        .nth(finding.line - 1)
                        .unwrap()
                        .contains("uses: actions/checkout"),
                    "{finding:?}"
                );
            }
            // File/line/next-line suppression filters definite bounds too; raw audit keeps them.
            let raw =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
            assert_eq!(
                raw.findings.len(),
                12 + 2 * usize::from(policy == Unanalyzable::Report)
            );
            assert_eq!(raw.advisories.len(), output.advisories.len());
            assert_eq!(sources.physical_read_count(), 11);
        }
    }
}

// Output cases moved to cli_gha_snapshot_checkout_credentials.
