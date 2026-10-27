use super::*;

// Finding cases moved to cli_pkgcfg_snapshot_package_json_registry_only.

#[test]
fn unreadable_package_json_is_unanalyzable() {
    let root = fixture("missing-input");
    let findings = check_package_json(&root.join("missing/package.json"), &root);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}

#[test]
fn configured_lockfile_reads_each_selected_project_once() {
    let root = fixture("project-lockfile-scope");
    let files = [
        root.join("web/package.json"),
        root.join("admin/package.json"),
    ];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for name in ["both-projects.yml", "web-only.yml", "excluded-admin.yml"] {
        let mut config = crate::config::v2::load_v2_config(&root, Some(&root.join(name))).unwrap();
        for policy in ["report", "advise", "ignore"] {
            config.rules[0].options =
                serde_yaml::from_str(&format!("lockfile: pnpm-lock.yaml\nunanalyzable: {policy}"))
                    .unwrap();
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
        }
    }
    // Selecting or excluding projects never repeats the same physical read.
    assert_eq!(sources.physical_read_count(), 4);
}
