use super::*;
use crate::config::v2::NoMistakesConfig;

fn wrapper_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queue-wrapper-unit-3904")
}

#[test]
fn enqueue_wrapper_config_preserves_positional_and_property_selectors() {
    let input = std::fs::read_to_string(wrapper_fixture().join(".no-mistakes.yml")).unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_str(&input).unwrap();
    let config: NoMistakesConfig = serde_yaml::from_str(&input).unwrap();
    let serialized = serde_yaml::to_value(config).unwrap();
    let roles = |value: &serde_yaml::Value| {
        let mut roles = value["queues"]["factories"]["enqueue"]
            .as_sequence()
            .unwrap()
            .clone();
        roles.sort_by_key(|role| role["callee"].as_str().unwrap().to_owned());
        roles
    };
    assert_eq!(roles(&serialized), roles(&yaml));
}

#[test]
fn enqueue_wrappers_resolve_const_aliases_and_use_effective_call_options() {
    let report = analyze_project(&wrapper_fixture(), None, &[]).unwrap();
    let mut sites: Vec<_> = report
        .producers
        .iter()
        .filter(|site| site.file == "enqueues.ts")
        .map(|site| (site.job.as_deref(), site.line))
        .collect();
    sites.sort_unstable();
    assert_eq!(
        sites,
        [
            (Some("aliased"), 8),
            (Some("aliased"), 9),
            (Some("legacy"), 28),
            (Some("unknown-options"), 24),
        ]
    );
    assert!(report
        .edges
        .iter()
        .any(|edge| { edge.from.ends_with("#aliased") && edge.to == "workers.ts" }));
    let invalid_ids: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.kind == "invalid-job-id")
        .map(|finding| finding.job.as_deref())
        .collect();
    assert_eq!(invalid_ids, [Some("aliased")]);
    assert!(report.check.iter().all(|finding| {
        !matches!(
            finding.kind.as_str(),
            "mixed-priority" | "missing-job-option"
        )
    }));
}

#[test]
fn enqueue_wrappers_abstain_on_direct_and_aliased_object_mutations() {
    let report = analyze_project(&wrapper_fixture(), None, &[]).unwrap();
    let mut jobs: Vec<_> = report
        .producers
        .iter()
        .filter(|site| site.file == "mutations.ts")
        .map(|site| site.job.as_deref())
        .collect();
    jobs.sort_unstable();
    assert_eq!(
        jobs,
        [
            Some("mutated-defaults"),
            Some("mutated-options"),
            Some("mutated-update"),
        ]
    );
    assert!(report
        .check
        .iter()
        .all(|finding| finding.file != "mutations.ts"));
}

#[test]
fn enqueue_wrapper_config_requires_exactly_one_queue_target() {
    for name in ["both-queue-targets", "neither-queue-target"] {
        let input = std::fs::read_to_string(
            wrapper_fixture()
                .join("configs")
                .join(format!("{name}.yml")),
        )
        .unwrap();
        let error = serde_yaml::from_str::<NoMistakesConfig>(&input).unwrap_err();
        assert!(
            error.to_string().contains(
                "enqueue factory `makeEnqueue` requires exactly one of queue and queueName"
            ),
            "{name}: {error}"
        );
    }
}

#[test]
fn enqueue_wrapper_config_rejects_unknown_property_selector_keys() {
    for field in ["queue", "job", "defaults", "options"] {
        let input = std::fs::read_to_string(
            wrapper_fixture()
                .join("configs")
                .join(format!("unknown-{field}-selector-key.yml")),
        )
        .unwrap();
        assert!(
            serde_yaml::from_str::<NoMistakesConfig>(&input).is_err(),
            "{field} selector accepted an unknown key"
        );
    }
}
