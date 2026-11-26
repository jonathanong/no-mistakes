use super::format::{missing_finding, unanalyzable_finding};
use super::judge::{call_arguments, judge_arguments, DelayVerdict};
use super::options::{collect_application_errors, validate, Options, Target, TargetInput};
use super::{check_with_catalog, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan, TsFileFacts};
use crate::codebase::ts_source::module_facts::arguments::{ArgumentValue, ArgumentValues};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, TypeScriptModuleFacts};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use crate::fx::FxHashMap;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn target(delay_argument: usize, delay_property: Option<&str>) -> Target {
    Target {
        module: "glide-mq".to_string(),
        export: "RateLimitError".to_string(),
        delay_argument,
        delay_property: delay_property.map(str::to_string),
    }
}

#[test]
fn options_validation_rejects_empty_blank_duplicate_and_out_of_range() {
    assert!(validate(&Options::default()).is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            module: " ".into(),
            export: "RateLimitError".into(),
            delay_argument: 0,
            delay_property: None,
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            module: "glide-mq".into(),
            export: "RateLimitError".into(),
            delay_argument: 32,
            delay_property: None,
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            module: "glide-mq".into(),
            export: "RateLimitError".into(),
            delay_argument: 0,
            delay_property: Some("  ".into()),
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![
            TargetInput {
                module: "glide-mq".into(),
                export: "RateLimitError".into(),
                delay_argument: 0,
                delay_property: None,
            },
            TargetInput {
                module: "glide-mq".into(),
                export: "RateLimitError".into(),
                delay_argument: 1,
                delay_property: None,
            },
        ],
    })
    .is_err());
}

fn config_messages(config: &NoMistakesConfig) -> Vec<String> {
    let mut errors = Vec::new();
    for rule in config
        .rules
        .iter()
        .filter(|rule| rule.enabled && rule.rule == RULE_ID)
    {
        if let Err(error) = config.effective_rule_analysis(rule) {
            errors.push(error.to_string());
        }
        match collect_application_errors(Path::new("/repo"), config, rule) {
            Ok(found) => errors.extend(found),
            Err(error) => errors.push(format!("{error:#}")),
        }
    }
    errors
}

#[test]
fn collect_config_errors_reports_unknown_fields_and_invalid_analysis_policy() {
    let bad_yaml = r#"
version: 2
rules:
  - rule: queue-rate-limit-error-delay
    scope: repository
    options:
      unanalyzable: bogus
      unknownField: true
      targets: []
"#;
    let config: NoMistakesConfig = serde_yaml::from_str(bad_yaml).unwrap();
    assert!(!config_messages(&config).is_empty());

    let valid_yaml = r#"
version: 2
rules:
  - rule: queue-rate-limit-error-delay
    scope: repository
    options:
      targets:
        - module: glide-mq
          export: RateLimitError
"#;
    let valid_config: NoMistakesConfig = serde_yaml::from_str(valid_yaml).unwrap();
    let ok_errors = config_messages(&valid_config);
    assert!(ok_errors.is_empty(), "{ok_errors:?}");
}

#[test]
fn judge_arguments_classifies_missing_positive_invalid_and_unanalyzable() {
    let pos0 = target(0, None);
    let empty = ArgumentValues {
        values: vec![],
        spread_from: None,
        path: None,
    };
    assert!(matches!(
        judge_arguments(Some(&empty), None, &pos0),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(None, None, &pos0),
        DelayVerdict::Unanalyzable(_)
    ));

    let spread = ArgumentValues {
        values: vec![ArgumentValue::Unknown],
        spread_from: Some(0),
        path: None,
    };
    assert!(matches!(
        judge_arguments(Some(&spread), None, &pos0),
        DelayVerdict::Unanalyzable(_)
    ));

    let positive = ArgumentValues {
        values: vec![ArgumentValue::Scalar(json!(5000))],
        spread_from: None,
        path: None,
    };
    assert_eq!(
        judge_arguments(Some(&positive), None, &pos0),
        DelayVerdict::Valid
    );

    let zero = ArgumentValues {
        values: vec![ArgumentValue::Scalar(json!(0))],
        spread_from: None,
        path: None,
    };
    assert!(matches!(
        judge_arguments(Some(&zero), None, &pos0),
        DelayVerdict::Missing(_)
    ));

    let dynamic_ok = ArgumentValues {
        values: vec![ArgumentValue::Unknown],
        spread_from: None,
        path: None,
    };
    assert_eq!(
        judge_arguments(Some(&dynamic_ok), None, &pos0),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(Some(&dynamic_ok), Some(&[(0, None)]), &pos0),
        DelayVerdict::Missing(_)
    ));

    let prop_target = target(0, Some("retryAfterMs"));
    let mut obj = FxHashMap::default();
    obj.insert(
        "retryAfterMs".to_string(),
        ArgumentValue::Scalar(json!(1500)),
    );
    let valid_obj = ArgumentValues {
        values: vec![ArgumentValue::Object(obj)],
        spread_from: None,
        path: None,
    };
    assert_eq!(
        judge_arguments(Some(&valid_obj), None, &prop_target),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(Some(&dynamic_ok), None, &prop_target),
        DelayVerdict::Unanalyzable(_)
    ));
    assert!(matches!(
        judge_arguments(Some(&dynamic_ok), Some(&[(0, None)]), &prop_target),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(Some(&positive), None, &prop_target),
        DelayVerdict::Missing(_)
    ));
    let mut bad_prop = FxHashMap::default();
    bad_prop.insert(
        "retryAfterMs".to_string(),
        ArgumentValue::Scalar(json!("soon")),
    );
    let bad_prop_obj = ArgumentValues {
        values: vec![ArgumentValue::Object(bad_prop)],
        spread_from: None,
        path: None,
    };
    assert!(matches!(
        judge_arguments(Some(&bad_prop_obj), None, &prop_target),
        DelayVerdict::Missing(_)
    ));
}

#[test]
fn call_arguments_and_format_and_entry_guards_handle_edge_cases() {
    let broken = TsFileFacts {
        parse_error: Some("syntax error".to_string()),
        ..TsFileFacts::default()
    };
    let broken_args = call_arguments(&broken, 0);
    assert!(broken_args.values.is_none() && broken_args.absent.is_none());

    let empty_facts = TsFileFacts::default();
    let empty_args = call_arguments(&empty_facts, 0);
    assert!(empty_args.values.is_none() && empty_args.absent.is_none());

    let app = RuleDef {
        rule: RULE_ID.to_string(),
        message: Some("custom prefix".to_string()),
        ..RuleDef::default()
    };
    let site = ResolvedCallSite {
        file: PathBuf::from("/repo/src/a.ts"),
        offset: 0,
        line: 3,
        caller: None,
        caller_id: None,
        source_callee: "RateLimitError".to_string(),
        invocation: InvocationKind::Construct,
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    let t = target(0, None);
    let missing = missing_finding(
        Path::new("/repo"),
        &app,
        Path::new("/repo/src/a.ts"),
        &site,
        &t,
        "omits delay",
    );
    assert!(missing.message.starts_with("custom prefix: "));
    let unan = unanalyzable_finding(
        Path::new("/repo"),
        &app,
        Path::new("/repo/src/a.ts"),
        1,
        None,
        "file could not be parsed",
    );
    assert!(unan.message.starts_with("custom prefix: "));
    assert!(unan.import.is_none());

    let empty_cfg = NoMistakesConfig::default();
    let empty_res = check_with_catalog(
        Path::new("/repo"),
        &empty_cfg,
        None,
        &CheckFactMap::default(),
        None,
    )
    .unwrap();
    assert!(empty_res.findings.is_empty());

    let active_cfg: NoMistakesConfig = serde_yaml::from_str(
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      targets:\n        - module: glide-mq\n          export: RateLimitError\n",
    )
    .unwrap();
    assert!(check_with_catalog(
        Path::new("/repo"),
        &active_cfg,
        None,
        &CheckFactMap::default(),
        None,
    )
    .is_err());
}

#[test]
fn absent_arg_projection_handles_signed_non_finite_const_objects_and_guards() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/queue-rate-limit-error-delay/absent-edge-cases.ts");
    let map = collect_ts_facts(
        std::slice::from_ref(&fixture),
        TsFactPlan {
            module_bindings: true,
            call_arguments: true,
            ..TsFactPlan::default()
        },
    );
    let facts = map.get(&fixture).expect("parsed fixture facts");
    let provenance = facts
        .module_bindings
        .as_deref()
        .and_then(|m| m.path_provenance.as_deref())
        .expect("path provenance");
    let entries: Vec<&Vec<(usize, Option<String>)>> =
        provenance.absent_or_non_finite_args.values().collect();
    // -Infinity, +NaN, and UNSET_OBJ ({ retryAfterMs: undefined })
    assert_eq!(entries.len(), 3, "{entries:?}");
    assert!(entries
        .iter()
        .any(|slots| slots.contains(&(0, Some("retryAfterMs".to_string())))));
}

#[test]
fn candidate_targets_parse_failures_and_config_errors_are_covered() {
    let module = TypeScriptModuleFacts {
        diagnostics: vec![ModuleDiagnostic {
            kind: "dynamic".to_string(),
            message: "eval".to_string(),
            span: None,
        }],
        ..TypeScriptModuleFacts::default()
    };
    let unsafe_facts = TsFileFacts {
        module_bindings: Some(Arc::new(module)),
        ..TsFileFacts::default()
    };
    let unsafe_args = call_arguments(&unsafe_facts, 0);
    assert!(unsafe_args.values.is_none() && unsafe_args.absent.is_none());

    for yaml in [
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      unanalyzable: bogus\n      targets:\n        - module: glide-mq\n          export: RateLimitError\n",
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      unknownField: 1\n      targets:\n        - module: glide-mq\n          export: RateLimitError\n",
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      targets: []\n",
    ] {
        let cfg: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(check_with_catalog(
            Path::new("/repo"),
            &cfg,
            None,
            &CheckFactMap::default(),
            None,
        )
        .is_err());
    }

    let broken_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/queue-rate-limit-error-delay/broken-parse");
    let findings = crate::codebase::rules::run_check(&broken_root, None, None).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "broken.ts");
    assert_eq!(findings[0].target.as_deref(), Some("unanalyzable"));
}
