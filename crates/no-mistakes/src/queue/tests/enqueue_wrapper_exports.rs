use super::*;
use crate::config::v2::NoMistakesConfig;

fn review_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/queue-wrapper-review-3904")
}

#[test]
fn wrapper_exports_resolve_stable_strings_aliases_and_queue_exports() {
    let report = analyze_project(&review_fixture(), None, &[]).unwrap();
    let mut producers: Vec<_> = report
        .producers
        .iter()
        .filter(|site| site.file == "positive.ts")
        .map(|site| (site.line, site.job.as_deref(), site.queue_name.as_deref()))
        .collect();
    producers.sort_unstable();
    assert_eq!(
        producers,
        [
            (8, Some("stable-job"), Some("review-queue")),
            (10, Some("stable-string"), Some("review-queue")),
            (12, Some("stable-string"), Some("review-queue")),
            (14, Some("stable-job"), Some("review-queue")),
            (16, Some("stable-string"), Some("review-queue")),
            (18, Some("stable-string"), Some("review-queue")),
            (20, Some("named-queue"), Some("review-queue")),
            (22, Some("queue-binding"), Some("review-queue")),
        ],
        "{report:#?}"
    );
    for job in [
        "stable-job",
        "stable-string",
        "named-queue",
        "queue-binding",
    ] {
        assert!(
            report.edges.iter().any(|edge| {
                edge.from == "positive.ts" && edge.to == format!("queues.ts#{job}")
            }),
            "{report:#?}"
        );
    }
}

#[test]
fn wrapper_exports_abstain_on_mutations_and_explicit_unresolved_shadows() {
    let report = analyze_project(&review_fixture(), None, &[]).unwrap();
    assert!(
        report
            .producers
            .iter()
            .filter(|site| site.file == "negative.ts")
            .all(|site| site.job.is_none() || site.queue_name.is_none()),
        "{report:#?}"
    );
    assert!(
        report
            .producers
            .iter()
            .filter(|site| site.file == "negative.ts")
            .filter_map(|site| site.job.as_deref())
            .all(|job| job == "mutable-queue-job"),
        "{report:#?}"
    );
    assert!(
        report.edges.iter().all(|edge| edge.from != "negative.ts"),
        "{report:#?}"
    );
    assert!(
        report
            .check
            .iter()
            .all(|finding| finding.file != "negative.ts"),
        "{report:#?}"
    );
    let legacy: Vec<_> = report
        .producers
        .iter()
        .filter(|site| site.file == "legacy.ts")
        .map(|site| (site.line, site.job.as_deref()))
        .collect();
    assert_eq!(legacy, [(5, Some("mutable-before"))], "{report:#?}");
}

#[test]
fn export_only_module_records_mutability_without_semantic_scoping() {
    let root = review_fixture();
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let path = root.join("mutable.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        crate::queue::extract::extract_program_with_scoping(
            &path,
            &source,
            program,
            &[],
            &config.queues.factories,
            false,
            None,
        )
    })
    .unwrap();
    assert_eq!(
        facts.exported_strings.get("STRING").map(String::as_str),
        Some("mutable-before")
    );
    assert!(!facts.stable_exported_strings.contains_key("STRING"));
    assert!(facts.blocked_enqueue_exports.contains("STRING"));
    assert!(facts.explicit_enqueue_exports.contains("STRING"));
}

#[test]
fn reassigned_const_string_keeps_legacy_value_but_blocks_stable_lookup() {
    let root = review_fixture();
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let path = root.join("const-reassigned.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        crate::queue::extract::extract_program_with_scoping(
            &path,
            &source,
            program,
            &[],
            &config.queues.factories,
            false,
            None,
        )
    })
    .unwrap();
    assert_eq!(
        facts.exported_strings.get("STRING").map(String::as_str),
        Some("const-before")
    );
    assert!(!facts.stable_exported_strings.contains_key("STRING"));
    assert!(facts.blocked_enqueue_exports.contains("STRING"));
}
