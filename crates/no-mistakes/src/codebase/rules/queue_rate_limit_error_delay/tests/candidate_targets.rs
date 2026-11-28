use super::{
    call_arguments, check_with_catalog, collect_ts_facts, CheckFactMap, InstancePropertyValue,
    NoMistakesConfig, TsFactPlan, TsFileFacts,
};
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, TypeScriptModuleFacts};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      unanalyzable: bogus\n      targets:\n        - module: glide-mq\n          export: Worker.RateLimitError\n          instanceDelayProperty: delayMs\n",
        "version: 2\nrules:\n  - rule: queue-rate-limit-error-delay\n    scope: repository\n    options:\n      unknownField: 1\n      targets:\n        - module: glide-mq\n          export: Worker.RateLimitError\n          instanceDelayProperty: delayMs\n",
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

    let inst_fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/queue-rate-limit-error-delay/instance-edge-cases.ts");
    let map = collect_ts_facts(
        std::slice::from_ref(&inst_fixture),
        TsFactPlan {
            module_bindings: true,
            call_arguments: true,
            ..TsFactPlan::default()
        },
    );
    let facts = map.get(&inst_fixture).expect("instance edge-case facts");
    let provenance = facts
        .module_bindings
        .as_deref()
        .and_then(|m| m.path_provenance.as_deref())
        .expect("path provenance");
    assert_eq!(provenance.call_derived_constructs.len(), 1);
    assert!(provenance
        .instance_properties
        .values()
        .any(|ev| ev.properties.get("delayMs") == Some(&InstancePropertyValue::Positive)));
    assert!(provenance
        .instance_properties
        .values()
        .any(|ev| ev.opaque_mutation));
}

#[test]
fn instance_review_tracks_throws_opaque_writes_and_assign_bindings() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/queue-rate-limit-error-delay/instance-review.ts");
    let map = collect_ts_facts(
        std::slice::from_ref(&path),
        TsFactPlan {
            module_bindings: true,
            call_arguments: true,
            ..TsFactPlan::default()
        },
    );
    let facts = map.get(&path).expect("instance review facts");
    let evidence: Vec<_> = facts
        .module_bindings
        .as_deref()
        .and_then(|module| module.path_provenance.as_deref())
        .expect("path provenance")
        .instance_properties
        .values()
        .collect();
    assert!(
        evidence
            .iter()
            .any(|ev| ev.properties.get("delayMs") == Some(&InstancePropertyValue::Invalid)),
        "assign after Object.assign should replace the delay, {evidence:?}"
    );
    assert!(
        evidence.iter().any(|ev| {
            ev.throw_count > 0
                && ev.properties.get("delayMs") == Some(&InstancePropertyValue::Positive)
                && !ev.positive_through_throws.contains("delayMs")
        }),
        "a throw before the delay is assigned stays unproven, {evidence:?}"
    );
    assert!(
        evidence.iter().any(|ev| {
            ev.throw_count > 0
                && !ev.opaque_mutation
                && ev.positive_through_throws.contains("delayMs")
        }),
        "a delay assigned before every throw stays proven, {evidence:?}"
    );
    assert!(
        evidence
            .iter()
            .any(|ev| ev.opaque_mutation && !ev.properties.contains_key("delayMs")),
        "opaque writes and helper calls drop the delay, {evidence:?}"
    );
    assert!(
        evidence.iter().any(|ev| {
            ev.opaque_mutation
                && ev.properties.get("delayMs") == Some(&InstancePropertyValue::Positive)
        }),
        "a later unconditional assignment restores a delay after a helper call, {evidence:?}"
    );
}
