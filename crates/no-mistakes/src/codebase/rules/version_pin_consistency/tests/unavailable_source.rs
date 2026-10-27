use super::*;

// Output cases moved to the cli_pkgcfg_snapshot modules.

fn saved_config(name: &str) -> NoMistakesConfig {
    let root = fixture("unavailable-source");
    crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap()
}

#[test]
fn cached_read_error_is_not_an_empty_source_or_a_second_read() {
    let root = fixture("unavailable-source");
    let mut cfg = saved_config("report.yml");
    cfg.rules[0].options["sourceFile"] = "invalid-utf8.toml".into();
    let files = vec![root.join("README.md")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let first = diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, true).unwrap();
    let second = diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, true).unwrap();
    assert_eq!(first.findings, second.findings);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn anchor_suppressions_apply_to_both_channels_and_defer_for_audit() {
    let root = fixture("unavailable-source");
    for name in ["disabled.md", "line.md"] {
        for policy in ["report", "advise"] {
            let mut cfg = saved_config(&format!("{policy}.yml"));
            cfg.rules[0].include = vec![name.into()];
            cfg.rules[0].options["anchors"][0]["file"] = name.into();
            let files = vec![root.join(name)];
            let sources = crate::codebase::rules::source_store_for_files(&files);
            let deferred =
                diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, true).unwrap();
            assert_eq!(deferred.findings.len() + deferred.advisories.len(), 1);
        }
    }
}

#[test]
fn unreadable_selected_anchor_routes_actual_error_through_every_policy() {
    let root = fixture("unavailable-source");
    let files = vec![root.join("invalid-utf8.md")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for policy in ["report", "advise", "ignore"] {
        let mut cfg = saved_config(&format!("{policy}.yml"));
        cfg.rules[0].include = vec!["invalid-utf8.md".into()];
        cfg.rules[0].options["sourceFile"] = "valid.toml".into();
        cfg.rules[0].options["anchors"][0]["file"] = "invalid-utf8.md".into();
        let first =
            diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, false).unwrap();
        let second =
            diagnostics_with_files_and_sources(&root, &cfg, &files, &sources, false).unwrap();
        assert_eq!(first.findings, second.findings, "{policy}");
        assert_eq!(first.advisories, second.advisories, "{policy}");
    }
    // The readable source and failed anchor are each read once across all checks.
    assert_eq!(sources.physical_read_count(), 2);
}
