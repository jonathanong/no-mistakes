use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};

// Output cases moved to cli_mrf_snapshot_markdown_reachability. Keep read
// errors, depth validation, lexical path helpers, and graph internals below.

mod depth_three;
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
    let mut plan = super::super::markdown_facts::MarkdownFactPlan::default();
    plan.request_pulldown(super::super::markdown_scope::markdown_files(&files));
    let facts = super::super::markdown_facts::MarkdownFactMap::prepare(&plan, &sources);
    check_with_files_sources_and_facts(root, config, &files, &facts)
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-reachability")
        .join(name)
}

#[test]
fn reports_prepared_read_failures_instead_of_treating_them_as_leaf_nodes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/markdown-read-failure");

    let error = run(
        &root,
        &config("rootFilenames: [unreadable.md]", &["**/*.md"], &[]),
        &["unreadable.md"],
    )
    .unwrap_err();

    assert!(error.to_string().contains(RULE_ID));
    assert!(error.to_string().contains("could not read Markdown file"));
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
    assert_eq!(validate_max_depth(Some(3)).unwrap(), 3);
    assert!(validate_max_depth(Some(4)).is_err());
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
fn finding_keys_normalize_relative_malformed_and_parent_only_paths() {
    assert_eq!(
        super::super::markdown_scope::finding_key(
            Path::new("/request"),
            Path::new("../relative.md"),
        ),
        "../relative.md"
    );
    assert_eq!(
        super::super::markdown_scope::finding_key(Path::new("request"), Path::new("//server")),
        "/server",
        "an incomplete UNC prefix falls back to a normalized POSIX path"
    );
    assert_eq!(
        super::super::markdown_scope::finding_key(Path::new("request"), Path::new("/../guide.md")),
        "/guide.md",
        "a parent component cannot escape an absolute filesystem root"
    );
    assert_eq!(
        super::super::markdown_scope::finding_key(Path::new("request"), Path::new("/")),
        "/"
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
