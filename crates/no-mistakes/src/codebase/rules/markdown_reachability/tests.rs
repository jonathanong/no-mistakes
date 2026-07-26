use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

mod scope_partition;

fn config(options: &str, include: &[&str], exclude: &[&str]) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            include: include.iter().map(|item| (*item).to_string()).collect(),
            exclude: exclude.iter().map(|item| (*item).to_string()).collect(),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(
    root: &Path,
    config: &NoMistakesConfig,
    relative_files: &[&str],
) -> Result<Vec<RuleFinding>> {
    let files = relative_files
        .iter()
        .map(|file| root.join(file))
        .collect::<Vec<_>>();
    let sources = super::super::source_store_for_files(&files);
    check_with_files_and_sources(root, config, &files, &sources)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-reachability")
        .join(name)
}
#[test]
fn resolves_only_links_inside_root() {
    let root = Path::new("/repo");
    assert_eq!(
        graph::normalize_inside(root, &root.join("a/../b.md")),
        Some(root.join("b.md"))
    );
    assert_eq!(
        graph::normalize_inside(root, &root.join("../../b.md")),
        None
    );
    assert_eq!(
        graph::normalize_inside(Path::new(""), Path::new(".")),
        Some(PathBuf::new())
    );
    assert_eq!(
        graph::normalize_inside(Path::new(""), Path::new("/b.md")),
        None
    );
}

#[test]
fn accepts_only_supported_depths() {
    assert_eq!(validate_max_depth(None).unwrap(), 2);
    assert_eq!(validate_max_depth(Some(1)).unwrap(), 1);
    assert!(validate_max_depth(Some(0)).is_err());
    assert!(validate_max_depth(Some(3)).is_err());
}

#[test]
fn full_check_accepts_direct_and_readme_paths_and_rejects_other_paths() {
    let root = fixture("paths");
    let files = [
        "CLAUDE.md",
        "README.md",
        "other.md",
        "direct.md",
        "indexed.md",
        "arbitrary.md",
        "lost.md",
    ];
    let findings = run(&root, &config("", &["**/*.md"], &[]), &files).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["arbitrary.md", "lost.md"]
    );
    assert_eq!(
        findings[0].message,
        "reachable at depth 2, but an intermediary must be a configured index Markdown file"
    );
    assert!(findings[1].message.contains("not reachable"));
}

#[test]
fn markdown_links_follow_the_actual_filesystem_case_resolution() {
    let root = fixture("case-path");
    let findings = run(
        &root,
        &config("", &["**/*.md"], &[]),
        &["CLAUDE.md", "guide.md"],
    )
    .unwrap();

    let candidate = root.join("Guide.MD");
    let tracked = root.join("guide.md");
    let case_variant_resolves = candidate.canonicalize().ok() == tracked.canonicalize().ok();
    if case_variant_resolves {
        assert!(findings.is_empty(), "Guide.MD resolves to tracked guide.md");
    } else {
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.file.as_str())
                .collect::<Vec<_>>(),
            ["guide.md"],
            "a case-sensitive filesystem keeps the Markdown link unresolved"
        );
    }
}

#[test]
fn reports_excess_depth_when_a_readme_path_exceeds_the_configured_limit() {
    let root = fixture("paths");
    let findings = run(
        &root,
        &config("maxDepth: 1", &["**/*.md"], &[]),
        &["CLAUDE.md", "README.md", "indexed.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "indexed.md");
    assert_eq!(
        findings[0].message,
        "reachable only at depth 2; maximum is 1"
    );
}

#[test]
fn rejects_duplicate_baseline_keys_across_external_project_roots() {
    let root = fixture("external-request");
    let mut config = config("rootFilenames: [ROOT.md]", &[], &[]);
    for name in ["external-one", "external-two"] {
        config.projects.insert(
            name.to_string(),
            crate::config::v2::schema::Project {
                root: Some(
                    root.parent()
                        .unwrap()
                        .join(name)
                        .to_string_lossy()
                        .to_string(),
                ),
                ..Default::default()
            },
        );
    }
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["external-one".to_string(), "external-two".to_string()];
    let error = run(
        &root,
        &config,
        &["../external-one/CLAUDE.md", "../external-two/CLAUDE.md"],
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("ambiguous baseline key `CLAUDE.md`"));
}

#[test]
fn baseline_finding_key_preserves_an_unresolvable_key() {
    let root = Path::new("/repo");
    let key = "../outside.md";
    assert_eq!(
        super::super::markdown_scope::baseline_finding_key(
            root,
            &[root.join("docs")],
            key,
            RULE_ID,
        )
        .unwrap(),
        key
    );
}

#[test]
fn finding_keys_handle_windows_volumes_without_host_specific_path_parsing() {
    assert_eq!(
        super::super::markdown_scope::finding_key(
            Path::new(r"C:\repo\docs"),
            Path::new(r"C:\repo\guides\..\guide.md"),
        ),
        "../guide.md"
    );
    assert_eq!(
        super::super::markdown_scope::finding_key(
            Path::new(r"C:\repo"),
            Path::new(r"D:\external\docs\..\guide.md"),
        ),
        "D:/external/guide.md",
        "cross-volume paths are external absolute findings, not parent traversals"
    );
    assert_eq!(
        super::super::markdown_scope::finding_key(
            Path::new(r"\\RequestHost\RequestShare\repo"),
            Path::new(r"\\ExternalHost\ExternalShare\docs\..\guide.md"),
        ),
        "//ExternalHost/ExternalShare/guide.md"
    );
    assert_eq!(
        super::super::markdown_scope::finding_key(
            Path::new(r"\\HOST\Share\repo"),
            Path::new(r"\\host\share\guide.md"),
        ),
        "../guide.md",
        "UNC roots compare case-insensitively but preserve emitted spelling"
    );
}

#[test]
fn default_depth_two_distinguishes_an_invalid_intermediary_in_findings_and_baselines() {
    let root = fixture("invalid-intermediary");
    let files = ["CLAUDE.md", "overview.md", "detail.md", "baseline.json"];
    let findings = run(&root, &config("", &["**/*.md"], &[]), &files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "detail.md");
    assert_eq!(
        findings[0].message,
        "reachable at depth 2, but an intermediary must be a configured index Markdown file"
    );
    assert!(run(
        &root,
        &config("baselineFile: baseline.json", &["**/*.md"], &[]),
        &files,
    )
    .unwrap()
    .is_empty());
    assert!(run(
        &root,
        &config("indexFilenames: [overview.md]", &["**/*.md"], &[]),
        &files,
    )
    .unwrap()
    .is_empty());
}

#[test]
fn recognizes_reference_links_and_ignores_non_local_or_escaping_links() {
    let root = fixture("links");
    let findings = run(
        &root,
        &config("", &["**/*.md"], &[]),
        &[
            "CLAUDE.md",
            "docs/doc.md",
            "docs/My Guide.md",
            "docs/unlinked.md",
            "docs/blocked.md",
        ],
    )
    .unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["docs/blocked.md", "docs/unlinked.md"]
    );
}

#[test]
fn shortest_depths_use_a_single_multi_source_bfs_and_skip_duplicate_queue_entries() {
    let root = Path::new("/repo");
    let claude = root.join("CLAUDE.md");
    let second_root = root.join("SECOND.md");
    let shared = root.join("shared.md");
    let target = root.join("target.md");
    let graph = BTreeMap::from([
        (claude.clone(), vec![shared.clone(), shared.clone()]),
        (second_root.clone(), vec![target.clone()]),
        (shared, vec![target.clone()]),
        (target.clone(), Vec::new()),
    ]);
    assert_eq!(
        graph::shortest_depths(
            &BTreeSet::from(["CLAUDE.md".to_string(), "SECOND.md".to_string()]),
            &graph,
        ),
        BTreeMap::from([
            (claude, 0),
            (second_root, 0),
            (root.join("shared.md"), 1),
            (target, 1),
        ])
    );
}

#[test]
fn baseline_requires_exact_state_and_filtered_targets_make_entries_stale() {
    let root = fixture("baseline-match");
    let options = "baselineFile: baseline.json";
    let files = ["CLAUDE.md", "other.md", "deep.md", "baseline.json"];
    let findings = run(&root, &config(options, &["**/*.md"], &[]), &files).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
    let filtered = run(&root, &config(options, &["**/*.md"], &["deep.md"]), &files).unwrap();
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].message.contains("deleted or excluded"));
}

#[test]
fn baseline_entry_for_a_configured_root_is_stale() {
    let root = fixture("baseline-root");
    let findings = run(
        &root,
        &config("baselineFile: baseline.json", &["**/*.md"], &[]),
        &["CLAUDE.md", "baseline.json"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("deleted or excluded"));
}

#[test]
fn mismatched_baseline_state_is_stale() {
    let root = fixture("baseline-mismatched");
    let findings = run(
        &root,
        &config("baselineFile: baseline.json", &["**/*.md"], &[]),
        &["CLAUDE.md", "other.md", "deep.md", "baseline.json"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("does not match"));
}

#[test]
fn baseline_must_be_tracked_and_valid_json() {
    let root = fixture("baseline-invalid");
    let config = config("baselineFile: baseline.json", &["**/*.md"], &[]);
    assert!(run(&root, &config, &["CLAUDE.md", "baseline.json"]).is_err());
    assert!(run(&root, &config, &["CLAUDE.md"])
        .unwrap_err()
        .to_string()
        .contains("tracked"));
}

#[test]
fn accepts_a_tracked_baseline_outside_the_rule_project_root() {
    let root = fixture("scoped");
    let mut config = config("baselineFile: baselines/reachability.json", &[], &[]);
    config.projects.insert(
        "docs".to_string(),
        crate::config::v2::schema::Project {
            root: Some("docs".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["docs".to_string()];
    let findings = run(
        &root,
        &config,
        &[
            "docs/CLAUDE.md",
            "docs/other.md",
            "docs/deep.md",
            "baselines/reachability.json",
        ],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn stale_external_baseline_entries_use_request_relative_finding_paths() {
    let root = fixture("external-request");
    let external = root.parent().unwrap().join("external-project");
    let mut config = config("baselineFile: baseline.json", &[], &[]);
    config.projects.insert(
        "external".to_string(),
        crate::config::v2::schema::Project {
            root: Some(external.to_string_lossy().to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["external".to_string()];
    let findings = run(
        &root,
        &config,
        &["baseline.json", "../external-project/CLAUDE.md"],
    )
    .unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "../external-project/stale.md");
}

#[test]
fn rejects_ambiguous_stale_external_baseline_keys() {
    let root = fixture("external-request");
    let mut config = config("baselineFile: baseline.json", &[], &[]);
    for name in ["external-one", "external-two"] {
        config.projects.insert(
            name.to_string(),
            crate::config::v2::schema::Project {
                root: Some(
                    root.parent()
                        .unwrap()
                        .join(name)
                        .to_string_lossy()
                        .to_string(),
                ),
                ..Default::default()
            },
        );
    }
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["external-one".to_string(), "external-two".to_string()];
    let error = run(
        &root,
        &config,
        &[
            "baseline.json",
            "../external-one/CLAUDE.md",
            "../external-two/CLAUDE.md",
        ],
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("ambiguous baseline key `stale.md`"));
}

#[test]
fn dispatcher_applies_standard_file_suppression() {
    let root = fixture("suppression");
    let config_path = root.join(".no-mistakes.yml");
    let findings = crate::codebase::rules::run_filesystem_rules_with_files(
        &root,
        Some(&config_path),
        &[root.join("CLAUDE.md"), root.join("lost.md")],
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}
