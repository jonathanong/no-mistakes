use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use super::options::{self, ApiTarget, Options};
use super::RULE_ID;

fn parse(yaml: &str) -> crate::config::v2::NoMistakesConfig {
    serde_yaml::from_str(yaml).unwrap()
}

fn configured(options: &str) -> crate::config::v2::NoMistakesConfig {
    parse(&format!(
        "rules:\n  - rule: {RULE_ID}\n    scope: repository\n    options:\n{options}"
    ))
}

fn saved(kind: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-single-enqueue-in-loop/fixtures")
        .join(kind)
}

fn marker(line: &str, label: &str) -> Option<String> {
    let token = format!("// {label}:");
    let index = line.find(&token)?;
    Some(line[index + token.len()..].trim().to_string())
}

type MarkerKey = (String, usize);
type MarkerReports = BTreeMap<MarkerKey, String>;
type MarkerSilence = BTreeSet<MarkerKey>;

fn markers(root: &Path) -> (MarkerReports, MarkerSilence) {
    let mut reports = BTreeMap::new();
    let mut silent = BTreeSet::new();
    for name in ["src/publish.ts", "src/ignored.ts"] {
        let text = std::fs::read_to_string(root.join(name)).unwrap();
        for (index, line) in text.lines().enumerate() {
            let key = (name.to_string(), index + 1);
            if let Some(construct) = marker(line, "report") {
                reports.insert(key.clone(), construct);
            }
            if marker(line, "silent").is_some() {
                silent.insert(key);
            }
        }
    }
    (reports, silent)
}

#[test]
fn empty_targets_do_not_demand_analysis_or_findings() {
    let config = configured("      single: []\n      bulk: []\n");
    assert!(!super::demands_analysis(&config));
    assert!(super::graph_plan(&config).is_none());
    assert!(!crate::codebase::rules::canonical_graph_requires_full_file_universe(&config));
    let findings = super::check(
        Path::new("/repo"),
        &config,
        None,
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn configured_targets_demand_calls_and_the_full_file_universe() {
    let config = configured("      single:\n        - {module: ./widgets, export: enqueueItem}\n");
    assert!(super::demands_analysis(&config));
    let plan = super::graph_plan(&config).unwrap();
    assert!(plan.calls);
    assert!(!plan.imports);
    assert!(crate::codebase::rules::canonical_graph_requires_full_file_universe(&config));
    let canonical = crate::codebase::rules::canonical_graph_plan(&config).unwrap();
    assert!(canonical.calls);
}

#[test]
fn invalid_options_demand_analysis_and_fail_before_the_graph() {
    let config = configured("      single: 1\n");
    assert!(super::demands_analysis(&config));
    let error = super::check(
        Path::new("/repo"),
        &config,
        None,
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("invalid options"), "{error:#}");
}

#[test]
fn unknown_option_fields_are_rejected() {
    let config = configured("      extra: true\n");
    let error = config.rules[0].try_rule_options::<Options>().unwrap_err();
    assert!(error.to_string().contains("extra"), "{error:#}");
}

#[test]
fn blank_duplicate_and_overlapping_targets_are_rejected() {
    let blank = Options {
        single: vec![ApiTarget {
            module: "  ".into(),
            export: "enqueueItem".into(),
        }],
        bulk: Vec::new(),
    };
    let error = options::validate(&blank).unwrap_err();
    assert!(error.to_string().contains("non-empty"), "{error:#}");

    let blank_export = Options {
        single: Vec::new(),
        bulk: vec![ApiTarget {
            module: "./widgets".into(),
            export: " ".into(),
        }],
    };
    let error = options::validate(&blank_export).unwrap_err();
    assert!(error.to_string().contains("options.bulk[0]"), "{error:#}");

    let duplicate = Options {
        single: vec![
            ApiTarget {
                module: " ./widgets ".into(),
                export: " enqueueItem ".into(),
            },
            ApiTarget {
                module: "./widgets".into(),
                export: "enqueueItem".into(),
            },
        ],
        bulk: Vec::new(),
    };
    let error = options::validate(&duplicate).unwrap_err();
    assert!(error.to_string().contains("more than once"), "{error:#}");

    let overlap = Options {
        single: vec![ApiTarget {
            module: "./widgets".into(),
            export: "enqueueItem".into(),
        }],
        bulk: vec![ApiTarget {
            module: "./widgets".into(),
            export: "enqueueItem".into(),
        }],
    };
    let error = options::validate(&overlap).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("both options.single and options.bulk"),
        "{error:#}"
    );
}

#[test]
fn trimmed_targets_keep_the_specifier_and_export() {
    let options = Options {
        single: vec![ApiTarget {
            module: " ./widgets ".into(),
            export: " enqueueItem ".into(),
        }],
        bulk: vec![ApiTarget {
            module: " ./widgets ".into(),
            export: " enqueueItems ".into(),
        }],
    };
    let targets = options::validate(&options).unwrap();
    assert_eq!(
        targets.single,
        vec![("./widgets".into(), "enqueueItem".into())]
    );
    assert_eq!(
        targets.bulk,
        vec![("./widgets".into(), "enqueueItems".into())]
    );
}

#[test]
fn a_missing_graph_fails_only_after_targets_validate() {
    let config = configured("      single:\n        - {module: ./widgets, export: enqueueItem}\n");
    let error = super::check(
        Path::new("/repo"),
        &config,
        None,
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("canonical call graph"),
        "{error:#}"
    );
}

#[test]
fn an_invalid_include_glob_is_reported() {
    let mut config =
        configured("      single:\n        - {module: ./widgets, export: enqueueItem}\n");
    config.rules[0].include = vec!["[".into()];
    let root = PathBuf::from("/repo");
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        HashMap::new(),
        HashMap::new(),
    );
    let error = super::check(
        &root,
        &config,
        Some(&graph),
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("include contains invalid glob"),
        "{error:#}"
    );
}

#[test]
fn files_without_typescript_facts_are_skipped() {
    let config = configured("      single:\n        - {module: ./widgets, export: enqueueItem}\n");
    let mut facts = crate::codebase::check_facts::CheckFactMap::default();
    facts.files.push(PathBuf::from("notes.md"));
    let root = PathBuf::from("/repo");
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        HashMap::new(),
        HashMap::new(),
    );
    let findings = super::check(&root, &config, Some(&graph), &facts).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn an_empty_application_is_skipped_beside_a_configured_one() {
    let config = parse(&format!(
        "rules:\n  - rule: {RULE_ID}\n    scope: repository\n    options:\n      single: []\n  - rule: {RULE_ID}\n    scope: repository\n    options:\n      single:\n        - {{module: ./widgets, export: enqueueItem}}\n"
    ));
    let root = PathBuf::from("/repo");
    let graph = crate::codebase::dependencies::graph::test_support::from_raw_maps(
        root.clone(),
        HashMap::new(),
        HashMap::new(),
    );
    let findings = super::check(
        &root,
        &config,
        Some(&graph),
        &crate::codebase::check_facts::CheckFactMap::default(),
    )
    .unwrap();
    assert!(findings.is_empty());
}

#[test]
fn saved_fixture_reports_only_the_marked_open_single_enqueues() {
    let source = saved("configured");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    let findings =
        crate::codebase::rules::run_check(root, Some(&root.join(".no-mistakes.yml")), None)
            .unwrap();
    let (reports, silent) = markers(root);
    let found: BTreeSet<_> = findings
        .iter()
        .map(|finding| (finding.file.clone(), finding.line))
        .collect();
    assert_eq!(found, reports.keys().cloned().collect(), "{findings:#?}");
    for key in &silent {
        assert!(!found.contains(key), "silent line was reported: {key:?}");
    }
    for finding in &findings {
        let construct = &reports[&(finding.file.clone(), finding.line)];
        assert!(
            finding.message.contains(&format!("inside {construct}")),
            "{finding:#?}"
        );
        assert_eq!(finding.rule, RULE_ID);
        assert_eq!(finding.target.as_deref(), Some("./widgets#enqueueItem"));
        let source_line = std::fs::read_to_string(root.join(&finding.file))
            .unwrap()
            .lines()
            .nth(finding.line - 1)
            .unwrap()
            .to_string();
        let import = if source_line.contains("queueWidget") {
            "queueWidget"
        } else if source_line.contains("widgets.enqueueItem") {
            "widgets.enqueueItem"
        } else {
            "enqueueItem"
        };
        assert_eq!(finding.import.as_deref(), Some(import), "{finding:#?}");
    }
}

#[test]
fn a_custom_message_prefixes_the_same_findings() {
    let source = saved("configured");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    let plain = crate::codebase::rules::run_check(root, Some(&root.join(".no-mistakes.yml")), None)
        .unwrap();
    let prefixed =
        crate::codebase::rules::run_check(root, Some(&root.join("message.yml")), None).unwrap();
    assert_eq!(plain.len(), prefixed.len());
    for finding in prefixed {
        assert!(
            finding
                .message
                .starts_with("Queue one batch for the collection: "),
            "{finding:#?}"
        );
    }
}

#[test]
fn empty_target_lists_stay_silent_on_a_single_enqueue_loop() {
    let source = saved("empty-targets");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path();
    let findings =
        crate::codebase::rules::run_check(root, Some(&root.join(".no-mistakes.yml")), None)
            .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}
