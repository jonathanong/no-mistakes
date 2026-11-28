use super::{analyze_project, fixture};
use crate::config::v2::NoMistakesConfig;
use crate::queue::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionSite, LocalQueue, QueueDefaultOptions,
    RequiredJobOptions,
};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[test]
fn unset_job_options_add_nothing() {
    let root = fixture("job-options-unset");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report
        .check
        .iter()
        .all(|finding| finding.kind != "missing-job-option"));
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.contains("job options")));
}

#[test]
fn required_job_options_round_trip_and_stay_off_the_default_document() {
    let yaml = r#"
version: 2
queues:
  requiredJobOptions: [attempts]
  requiredJobOptionsByQueue:
    items: [priority]
    orders: []
"#;
    let config: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(
        config.queues.required_job_options,
        vec!["attempts".to_string()]
    );
    assert_eq!(
        config.queues.required_job_options_by_queue["items"],
        vec!["priority".to_string()]
    );
    assert!(config.queues.required_job_options_by_queue["orders"].is_empty());
    let rendered = serde_yaml::to_string(&config).unwrap();
    assert!(rendered.contains("requiredJobOptions"));
    assert!(rendered.contains("orders: []"));
    let default_document = serde_yaml::to_string(&NoMistakesConfig::default()).unwrap();
    assert!(!default_document.contains("requiredJobOptions"));
}

#[test]
fn stamp_clears_previous_requirements_when_the_selected_config_is_empty() {
    let policy = RequiredJobOptions {
        global: vec!["attempts".to_string()],
        by_queue: BTreeMap::new(),
    };
    let mut facts = HashMap::new();
    let file = FileFacts {
        required_job_options: Some(policy.clone()),
        ..FileFacts::default()
    };
    facts.insert(PathBuf::from("widgets.ts"), file);
    crate::queue::job_options::stamp(&mut facts, &Default::default());
    assert!(facts
        .values()
        .next()
        .unwrap()
        .required_job_options
        .is_none());
}

#[test]
fn unresolved_queue_uses_the_global_list_when_no_queue_overrides_exist() {
    let mut facts = HashMap::new();
    let file = FileFacts {
        required_job_options: Some(RequiredJobOptions {
            global: vec!["attempts".to_string()],
            by_queue: BTreeMap::new(),
        }),
        job_option_sites: vec![JobOptionSite {
            line: 3,
            binding: None,
            queue_name: None,
            queue_file: None,
            declaration: None,
            ambiguous_defaults: false,
            job: Some("paint".to_string()),
            evidence: JobOptionEvidence::default(),
        }],
        ..FileFacts::default()
    };
    facts.insert(PathBuf::from("/tmp/widgets.ts"), file);
    facts.insert(PathBuf::from("/tmp/other.ts"), FileFacts::default());
    crate::queue::job_options::judge_into_facts(
        Path::new("/tmp"),
        &mut facts,
        &crate::queue::test_scope::ConfiguredTestScope::empty(),
    );
    let finding = &facts
        .get(Path::new("/tmp/widgets.ts"))
        .unwrap()
        .job_option_findings[0];
    assert_eq!(finding.message, "job options missing attempts");
    assert!(facts
        .get(Path::new("/tmp/other.ts"))
        .unwrap()
        .job_option_findings
        .is_empty());
}

#[test]
fn declaration_miss_and_imported_queue_default_still_report_the_global_key() {
    let path = PathBuf::from("/tmp/queues.ts");
    let file = FileFacts {
        required_job_options: Some(RequiredJobOptions {
            global: vec!["attempts".to_string()],
            by_queue: BTreeMap::new(),
        }),
        queue_declarations: HashMap::from([(2, LocalQueue::ImportedName("q".to_string()))]),
        queue_defaults: vec![QueueDefaultOptions {
            declaration: 2,
            line: 1,
            queue_name: None,
            evidence: JobOptionEvidence::default(),
        }],
        job_option_sites: vec![JobOptionSite {
            line: 9,
            binding: None,
            queue_name: None,
            queue_file: Some(path.clone()),
            declaration: Some(99),
            ambiguous_defaults: false,
            job: Some("paint".to_string()),
            evidence: JobOptionEvidence::default(),
        }],
        ..FileFacts::default()
    };
    let mut facts = HashMap::new();
    facts.insert(path.clone(), file);
    crate::queue::job_options::judge_into_facts(
        Path::new("/tmp"),
        &mut facts,
        &crate::queue::test_scope::ConfiguredTestScope::empty(),
    );
    let judged = facts.get(&path).unwrap();
    assert!(judged.diagnostics.is_empty(), "{:?}", judged.diagnostics);
    assert_eq!(judged.job_option_findings.len(), 1);
    assert_eq!(
        judged.job_option_findings[0].message,
        "job options missing attempts"
    );
}

#[test]
fn unresolved_receiver_abstains_without_queue_overrides() {
    let mut facts = HashMap::new();
    let file = FileFacts {
        required_job_options: Some(RequiredJobOptions {
            global: vec!["attempts".to_string()],
            by_queue: BTreeMap::new(),
        }),
        job_option_sites: vec![JobOptionSite {
            line: 4,
            binding: Some(crate::queue::extract_model::ProducerBinding::Named(
                "tags".to_string(),
            )),
            queue_name: None,
            queue_file: None,
            declaration: None,
            ambiguous_defaults: false,
            job: Some("red".to_string()),
            evidence: JobOptionEvidence::default(),
        }],
        ..FileFacts::default()
    };
    facts.insert(PathBuf::from("/tmp/enqueue.ts"), file);
    crate::queue::job_options::judge_into_facts(
        Path::new("/tmp"),
        &mut facts,
        &crate::queue::test_scope::ConfiguredTestScope::empty(),
    );
    let judged = facts.get(Path::new("/tmp/enqueue.ts")).unwrap();
    assert!(judged.job_option_findings.is_empty());
    assert!(judged.diagnostics.is_empty());
}

#[test]
fn each_rule_builds_its_allow_matcher_once_for_multiple_files() {
    use crate::config::v2::schema::queues_top::allow_paths_test_hook::take_builds;

    for name in ["unbounded-retention", "job-state-as-record"] {
        take_builds();
        analyze_project(&fixture(name), None, &[]).unwrap();
        assert_eq!(take_builds(), 1, "{name}");
    }
}

#[test]
fn empty_and_disabled_allow_lists_build_no_matcher() {
    use crate::config::v2::schema::queues_top::allow_paths_test_hook::take_builds;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check");
    for rule in ["queue-retention-config", "queue-job-state-config"] {
        for mode in ["default-on", "default-off"] {
            take_builds();
            analyze_project(&root.join(rule).join(mode), None, &[]).unwrap();
            assert_eq!(take_builds(), 0, "{rule}/{mode}");
        }
    }
}
