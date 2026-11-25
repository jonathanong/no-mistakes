use super::{analyze_project, fixture};
use crate::config::v2::NoMistakesConfig;
use crate::queue::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionSite, RequiredJobOptions,
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
    crate::queue::job_options::judge_into_facts(Path::new("/tmp"), &mut facts);
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
    crate::queue::job_options::judge_into_facts(Path::new("/tmp"), &mut facts);
    let judged = facts.get(Path::new("/tmp/enqueue.ts")).unwrap();
    assert!(judged.job_option_findings.is_empty());
    assert!(judged.diagnostics.is_empty());
}

#[test]
fn stamps_each_compiled_allow_policy_once_for_all_files() {
    use crate::config::v2::schema::{
        JobStateAsRecordConfig, QueuesTopLevelConfig, UnboundedJobRetentionConfig,
    };
    use std::sync::Arc;

    let queues = QueuesTopLevelConfig {
        unbounded_job_retention: Some(UnboundedJobRetentionConfig {
            allow_paths: vec!["retention/**".into()],
            ..UnboundedJobRetentionConfig::all()
        }),
        job_state_as_record: Some(JobStateAsRecordConfig {
            allow_paths: vec!["state/**".into()],
        }),
        ..QueuesTopLevelConfig::default()
    };
    let mut facts = HashMap::from([
        (PathBuf::from("retention/first.ts"), FileFacts::default()),
        (PathBuf::from("state/second.ts"), FileFacts::default()),
    ]);

    crate::queue::job_options::stamp(&mut facts, &queues);

    let first = &facts[Path::new("retention/first.ts")];
    let second = &facts[Path::new("state/second.ts")];
    let retention = first.unbounded_job_retention.as_ref().unwrap();
    assert!(Arc::ptr_eq(
        retention,
        second.unbounded_job_retention.as_ref().unwrap(),
    ));
    assert!(retention
        .allow
        .as_ref()
        .unwrap()
        .is_match("retention/first.ts"));
    let state = first.job_state_as_record.as_ref().unwrap();
    assert!(Arc::ptr_eq(
        state,
        second.job_state_as_record.as_ref().unwrap()
    ));
    assert!(state.is_match("state/second.ts"));
}
