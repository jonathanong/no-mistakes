use super::*;
use crate::queue::extract::ProducerSite;
use crate::queue::graph_model::{InternalProducer, InternalWorker};
use crate::queue::types::QueueKey;

fn site(worker_index: Option<usize>) -> ProducerSite {
    ProducerSite {
        file: PathBuf::from("/repo/workers/side.ts"),
        line: 4,
        binding: "leaf".to_string(),
        job: Some("ping".to_string()),
        job_import: None,
        raw_job: None,
        worker_index,
        self_dedup: false,
    }
}

#[test]
fn shared_file_workers_and_other_queue_dedup_are_not_findings() {
    let report = analyze_project(&fixture("factory-roles-false-positives"), None, &[]).unwrap();
    assert!(
        !report
            .check
            .iter()
            .any(|finding| finding.kind == "queue-cycle"),
        "two workers in one file should not share enqueues, check: {:?}",
        report.check
    );
    assert!(
        !report.check.iter().any(|finding| {
            finding.kind == "self-dedup-enqueue" && finding.file.contains("other-queue-dedup")
        }),
        "an add on another queue should not be self-dedup, check: {:?}",
        report.check
    );
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "self-dedup-enqueue"
                && finding.file.contains("same-queue-dedup")
                && finding.job.as_deref() == Some("x")
        }),
        "the same queue should still report self-dedup, check: {:?}",
        report.check
    );
    assert!(
        !report.check.iter().any(|finding| {
            finding.kind == "self-dedup-enqueue" && finding.job.as_deref() == Some("y")
        }),
        "a same-line addBulk that records nothing should not mark the previous add, check: {:?}",
        report.check
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.file.contains("same-queue-dedup")
                && diagnostic
                    .message
                    .contains("was not judged because the worker queue or the target queue")
        }),
        "an unresolved target should abstain, diagnostics: {:?}",
        report.diagnostics
    );
}

#[test]
fn top_level_enqueue_beside_workers_does_not_create_a_queue_cycle() {
    let report = analyze_project(&fixture("factory-roles-false-positives"), None, &[]).unwrap();
    assert!(report.producers.iter().any(|producer| {
        producer.file.ends_with("top-level-add.mts") && producer.job.as_deref() == Some("x")
    }));
    assert!(
        !report
            .check
            .iter()
            .any(|finding| finding.kind == "queue-cycle"),
        "a top-level enqueue beside two workers must not be attributed to either processor: {:?}",
        report.check
    );
}

#[test]
fn cycle_edges_skip_a_missing_worker_and_an_unresolved_worker_queue() {
    let root = PathBuf::from("/repo");
    let file = PathBuf::from("/repo/workers/side.ts");
    let queues = root.join("queues.ts");
    let target = Some(QueueKey {
        queue_file: queues.clone(),
        queue_name: "leaf".to_string(),
    });
    let producers = vec![
        InternalProducer {
            site: site(Some(3)),
            queue: target.clone(),
        },
        InternalProducer {
            site: site(Some(0)),
            queue: target,
        },
    ];
    let workers = vec![InternalWorker {
        site: crate::queue::extract::WorkerSite {
            file,
            line: 2,
            queue_name: None,
            queue_name_import: None,
            jobs: vec!["ping".to_string()],
            processor_specifier: None,
            processor_file: None,
            wildcard: false,
        },
        queue: None,
    }];
    let findings = super::super::cycles::queue_cycles(&root, &producers, &workers);
    assert!(
        findings.is_empty(),
        "an enqueue is not a cycle edge without its worker queue: {findings:?}"
    );
}
