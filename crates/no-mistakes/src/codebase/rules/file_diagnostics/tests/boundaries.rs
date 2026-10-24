use super::*;

#[test]
fn findings_only_filesystem_dispatch_honors_policy_and_file_suppression() {
    let root = saved();
    for &(rule, uncertainty) in CASES
        .iter()
        .filter(|(rule, _)| *rule != "server-route-client-boundary")
    {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            let mut config = config(&root, rule);
            config.unanalyzable = Some(policy);
            let files = files(&root, &config);
            let findings =
                super::super::super::filesystem_dispatch::run_filesystem_rules_with_config(
                    &root, &config, &files,
                )
                .unwrap();
            assert_eq!(
                findings.len(),
                1 + usize::from(policy == Unanalyzable::Report) * uncertainty,
                "{rule}: {findings:?}"
            );
            assert!(findings
                .iter()
                .all(|finding| !finding.file.contains("suppressed")));
        }
    }
}

#[test]
fn required_source_config_errors_fail_before_reading_the_selected_files() {
    let root = saved();
    type Engine = fn(
        &Path,
        &NoMistakesConfig,
        &[PathBuf],
        &SourceStore,
        bool,
    ) -> anyhow::Result<AnalysisDiagnostics>;
    let engines: &[(&str, Engine)] = &[
        ("no-empty-or-comments-only-files", super::super::super::no_empty_or_comments_only_files::diagnostics_with_files_and_sources),
        ("structured-config-policy", super::super::super::structured_config_policy::diagnostics_with_files_and_sources),
    ];
    for &(rule, engine) in engines {
        for invalid in ["options", "include"] {
            let mut config = config(&root, rule);
            let files = files(&root, &config);
            if invalid == "options" {
                config.rules[0].options = true.into();
            } else {
                config.rules[0].include = vec!["[".into()];
            }
            let sources = super::super::super::source_store_for_files(&files);
            assert!(
                engine(&root, &config, &files, &sources, false).is_err(),
                "{rule}/{invalid}"
            );
            assert_eq!(sources.physical_read_count(), 0, "{rule}/{invalid}");
        }
    }
    let mut config = config(&root, "structured-config-policy");
    let files = files(&root, &config);
    config.rules[0].options["policies"][0]["files"] = serde_yaml::to_value(["["]).unwrap();
    let sources = super::super::super::source_store_for_files(&files);
    assert!(
        super::super::super::structured_config_policy::diagnostics_with_files_and_sources(
            &root, &config, &files, &sources, false
        )
        .is_err()
    );
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn boundary_parse_error_messages_use_repository_relative_paths() {
    let root = saved();
    let config = config(&root, "server-route-client-boundary");
    let files = files(&root, &config);
    let sources = super::super::super::source_store_for_files(&files);
    let prepared = evaluate(&root, &config, &files, &sources, false).findings;
    let standalone =
        super::super::super::server_route_client_boundary::check(&root, &config).unwrap();
    assert!(standalone
        .iter()
        .find(|finding| finding.file == "boundary/api/malformed.ts")
        .unwrap()
        .message
        .contains("boundary/api/malformed.ts"));
    for findings in [prepared, standalone] {
        let finding = findings
            .iter()
            .find(|finding| finding.file == "boundary/api/malformed.ts")
            .unwrap();
        assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
        assert!(
            !finding.message.contains(root.to_str().unwrap()),
            "{}",
            finding.message
        );
    }
}
