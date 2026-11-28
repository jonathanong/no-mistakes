use super::format::{missing_finding, unanalyzable_finding};
use super::judge::{call_arguments, judge_arguments, CallDelayArgs, DelayVerdict};
use super::options::{
    collect_application_errors, validate, DelaySpec, Options, Target, TargetInput,
};
use super::{check_with_catalog, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::InvocationKind;
use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan, TsFileFacts};
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentValue, ArgumentValues, InstancePropertyEvidence, InstancePropertyValue,
};
use crate::config::v2::schema::{NoMistakesConfig, RuleDef};
use crate::fx::FxHashMap;
use serde_json::json;
use std::path::{Path, PathBuf};

mod candidate_targets;

fn arg_target(delay_argument: usize, delay_property: Option<&str>) -> Target {
    Target {
        module: "glide-mq".to_string(),
        export: "GroupRateLimitError".to_string(),
        delay: DelaySpec::Argument {
            index: delay_argument,
            property: delay_property.map(str::to_string),
        },
    }
}

fn instance_target(prop: &str) -> Target {
    Target {
        module: "glide-mq".to_string(),
        export: "Worker.RateLimitError".to_string(),
        delay: DelaySpec::InstanceProperty(prop.to_string()),
    }
}

fn call_args<'a>(
    values: Option<&'a ArgumentValues>,
    absent: Option<&'a [(usize, Option<String>)]>,
    instance: Option<&'a InstancePropertyEvidence>,
) -> CallDelayArgs<'a> {
    CallDelayArgs {
        values,
        absent,
        instance,
        call_derived_construct: false,
        dynamic_scope: false,
    }
}

#[test]
fn options_validation_rejects_empty_blank_duplicate_conflicts_and_out_of_range() {
    let base = |module: &str, export: &str| TargetInput {
        module: module.into(),
        export: export.into(),
        delay_argument: None,
        delay_property: None,
        instance_delay_property: None,
    };
    assert!(validate(&Options::default()).is_err());
    assert!(validate(&Options {
        targets: vec![base(" ", "GroupRateLimitError")],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            delay_argument: Some(32),
            ..base("glide-mq", "GroupRateLimitError")
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            delay_property: Some("  ".into()),
            ..base("glide-mq", "GroupRateLimitError")
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            instance_delay_property: Some("  ".into()),
            ..base("glide-mq", "Worker.RateLimitError")
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![TargetInput {
            delay_argument: Some(0),
            instance_delay_property: Some("delayMs".into()),
            ..base("glide-mq", "Worker.RateLimitError")
        }],
    })
    .is_err());
    assert!(validate(&Options {
        targets: vec![
            base("glide-mq", "GroupRateLimitError"),
            TargetInput {
                delay_argument: Some(1),
                ..base("glide-mq", "GroupRateLimitError")
            },
        ],
    })
    .is_err());
    let valid = validate(&Options {
        targets: vec![TargetInput {
            instance_delay_property: Some("delayMs".into()),
            ..base("glide-mq", "Worker.RateLimitError")
        }],
    })
    .unwrap();
    assert_eq!(
        valid[0].delay,
        DelaySpec::InstanceProperty("delayMs".into())
    );
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
          export: Worker.RateLimitError
          instanceDelayProperty: delayMs
"#;
    let valid_config: NoMistakesConfig = serde_yaml::from_str(valid_yaml).unwrap();
    let ok_errors = config_messages(&valid_config);
    assert!(ok_errors.is_empty(), "{ok_errors:?}");
}

fn av(values: Vec<ArgumentValue>) -> ArgumentValues {
    ArgumentValues {
        values,
        spread_from: None,
        path: None,
    }
}

fn ev(key: &str, value: InstancePropertyValue) -> InstancePropertyEvidence {
    let mut properties = FxHashMap::default();
    properties.insert(key.to_string(), value);
    InstancePropertyEvidence {
        properties,
        ..InstancePropertyEvidence::default()
    }
}

#[test]
fn judge_arguments_classifies_positional_object_and_instance_property_targets() {
    let pos0 = arg_target(0, None);
    let empty = av(vec![]);
    let spread = ArgumentValues {
        values: vec![ArgumentValue::Unknown],
        spread_from: Some(0),
        path: None,
    };
    let positive = av(vec![ArgumentValue::Scalar(json!(5000))]);
    let zero = av(vec![ArgumentValue::Scalar(json!(0))]);
    let dynamic_ok = av(vec![ArgumentValue::Unknown]);
    let non_obj_arr = av(vec![ArgumentValue::Array(vec![])]);
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, None), &pos0),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(None, None, None), &pos0),
        DelayVerdict::Unanalyzable(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&spread), None, None), &pos0),
        DelayVerdict::Unanalyzable(_)
    ));
    assert_eq!(
        judge_arguments(&call_args(Some(&positive), None, None), &pos0),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(&call_args(Some(&zero), None, None), &pos0),
        DelayVerdict::Missing(_)
    ));
    assert_eq!(
        judge_arguments(&call_args(Some(&dynamic_ok), None, None), &pos0),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(
            &call_args(Some(&dynamic_ok), Some(&[(0, None)]), None),
            &pos0
        ),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&non_obj_arr), None, None), &pos0),
        DelayVerdict::Missing(_)
    ));

    let prop_target = arg_target(0, Some("retryAfterMs"));
    let obj_with = |key: &str, value: ArgumentValue| {
        let mut map = FxHashMap::default();
        map.insert(key.to_string(), value);
        av(vec![ArgumentValue::Object(map)])
    };
    let valid_obj = obj_with("retryAfterMs", ArgumentValue::Scalar(json!(1500)));
    let bad_prop_obj = obj_with("retryAfterMs", ArgumentValue::Scalar(json!("soon")));
    let dyn_prop_obj = obj_with("retryAfterMs", ArgumentValue::Unknown);
    let other_prop_obj = obj_with("other", ArgumentValue::Scalar(json!(1500)));
    assert_eq!(
        judge_arguments(&call_args(Some(&valid_obj), None, None), &prop_target),
        DelayVerdict::Valid
    );
    assert_eq!(
        judge_arguments(&call_args(Some(&dyn_prop_obj), None, None), &prop_target),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(
            &call_args(
                Some(&dyn_prop_obj),
                Some(&[(0, Some("retryAfterMs".to_string()))]),
                None,
            ),
            &prop_target,
        ),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, None), &prop_target),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&other_prop_obj), None, None), &prop_target),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&dynamic_ok), None, None), &prop_target),
        DelayVerdict::Unanalyzable(_)
    ));
    assert!(matches!(
        judge_arguments(
            &call_args(Some(&dynamic_ok), Some(&[(0, None)]), None),
            &prop_target
        ),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&positive), None, None), &prop_target),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&bad_prop_obj), None, None), &prop_target),
        DelayVerdict::Missing(_)
    ));

    let inst = instance_target("delayMs");
    let good_ev = ev("delayMs", InstancePropertyValue::Positive);
    let inv_ev = ev("delayMs", InstancePropertyValue::Invalid);
    let abs_ev = ev("delayMs", InstancePropertyValue::AbsentOrNonFinite);
    let other_ev = ev("code", InstancePropertyValue::Invalid);
    let opaque_ev = InstancePropertyEvidence {
        opaque_mutation: true,
        ..InstancePropertyEvidence::default()
    };
    let mut thrown_without_delay = ev("delayMs", InstancePropertyValue::Positive);
    thrown_without_delay.throw_count = 1;
    let mut thrown_with_delay = ev("delayMs", InstancePropertyValue::Positive);
    thrown_with_delay.throw_count = 1;
    thrown_with_delay
        .positive_through_throws
        .insert("delayMs".to_string());
    assert!(matches!(
        judge_arguments(&call_args(None, None, None), &inst),
        DelayVerdict::Unanalyzable(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, None), &inst),
        DelayVerdict::Missing(_)
    ));
    assert_eq!(
        judge_arguments(&call_args(Some(&empty), None, Some(&good_ev)), &inst),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, Some(&inv_ev)), &inst),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, Some(&abs_ev)), &inst),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, Some(&other_ev)), &inst),
        DelayVerdict::Missing(_)
    ));
    assert!(matches!(
        judge_arguments(&call_args(Some(&empty), None, Some(&opaque_ev)), &inst),
        DelayVerdict::Unanalyzable(_)
    ));
    assert!(matches!(
        judge_arguments(
            &call_args(Some(&empty), None, Some(&thrown_without_delay)),
            &inst
        ),
        DelayVerdict::Missing(_)
    ));
    assert_eq!(
        judge_arguments(
            &call_args(Some(&empty), None, Some(&thrown_with_delay)),
            &inst
        ),
        DelayVerdict::Valid
    );
    assert!(matches!(
        judge_arguments(
            &CallDelayArgs {
                values: Some(&empty),
                absent: None,
                instance: None,
                call_derived_construct: false,
                dynamic_scope: true,
            },
            &inst,
        ),
        DelayVerdict::Unanalyzable(_)
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
        source_callee: "Worker.RateLimitError".to_string(),
        invocation: InvocationKind::Construct,
        target: ResolvedCallTarget::Unknown,
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    let t = instance_target("delayMs");
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
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      targets:\n        - module: glide-mq\n          export: Worker.RateLimitError\n          instanceDelayProperty: delayMs\n",
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
    assert_eq!(entries.len(), 3, "{entries:?}");
    assert!(entries
        .iter()
        .any(|slots| slots.contains(&(0, Some("retryAfterMs".to_string())))));
}
