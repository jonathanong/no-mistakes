use super::judge::{judge_arguments, DelayVerdict};
use super::options::{validate, Options, Target, TargetInput};
use crate::codebase::ts_source::module_facts::arguments::{ArgumentValue, ArgumentValues};
use crate::fx::FxHashMap;
use serde_json::json;

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
}
