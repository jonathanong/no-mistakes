use crate::queue::extract::FileFacts;
use crate::queue::extract_model::EnqueuePriority;
use crate::queue::graph_model::{CheckFinding, InternalProducer};
use crate::queue::source::relative_string;
use crate::queue::test_scope::ConfiguredTestScope;
use crate::queue::types::{Diagnostic, DiagnosticKind, QueueKey, Severity};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[path = "priority_flow.rs"]
mod flow;

use flow::{flow_queue_definitions, opaque_queues, resolved_flow_queue};

pub(super) fn mixed_priority_findings(
    root: &Path,
    producers: &[InternalProducer],
    facts: &HashMap<PathBuf, FileFacts>,
    test_scope: &ConfiguredTestScope,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<CheckFinding> {
    let producers = producers
        .iter()
        .filter(|producer| !test_scope.is_match(root, &producer.site.file))
        .collect::<Vec<_>>();
    let flow_queues = flow_queue_definitions(root, facts, &producers, test_scope);
    let opaque = opaque_queues(root, facts, &flow_queues, test_scope);
    let mut groups: HashMap<QueueKey, Vec<&InternalProducer>> = HashMap::new();
    for producer in producers {
        let Some(queue) = producer.queue.as_ref() else {
            continue;
        };
        let definition = if facts.contains_key(&producer.site.file) {
            flow_queues.get(queue.queue_name.as_str())
        } else {
            None
        };
        let Some(queue) = resolved_flow_queue(queue, definition) else {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                file: relative_string(root, &producer.site.file),
                line: producer.site.line,
                message: format!(
                    "mixed-priority was not judged for queue {:?} because the flow queue has multiple definition files",
                    queue.queue_name
                ),
                kind: DiagnosticKind::CheckAbstention,
            });
            continue;
        };
        groups.entry(queue).or_default().push(producer);
    }
    let mut findings = Vec::new();
    for (queue, group) in groups {
        if opaque.contains(&queue) {
            continue;
        }
        findings.extend(report_queue(root, &queue, &group));
    }
    findings
}

fn report_queue(root: &Path, queue: &QueueKey, group: &[&InternalProducer]) -> Vec<CheckFinding> {
    if group
        .iter()
        .any(|producer| producer.site.priority == EnqueuePriority::Dynamic)
    {
        return Vec::new();
    }
    let mut plain = Vec::new();
    let mut ranked = Vec::new();
    for producer in group {
        match producer.site.priority {
            EnqueuePriority::Unprioritized => plain.push(*producer),
            EnqueuePriority::Prioritized(_) => ranked.push(*producer),
            EnqueuePriority::Dynamic => {}
        }
    }
    if plain.is_empty() || ranked.is_empty() {
        return Vec::new();
    }
    let mut findings = Vec::new();
    if plain.len() <= ranked.len() {
        findings.extend(emit(root, queue, &plain, &ranked));
    }
    if ranked.len() <= plain.len() {
        findings.extend(emit(root, queue, &ranked, &plain));
    }
    findings
}

fn emit(
    root: &Path,
    queue: &QueueKey,
    reported: &[&InternalProducer],
    others: &[&InternalProducer],
) -> Vec<CheckFinding> {
    let others = ordered(others);
    reported
        .iter()
        .map(|producer| CheckFinding {
            kind: "mixed-priority".to_string(),
            file: relative_string(root, &producer.site.file),
            line: producer.site.line,
            queue_file: Some(relative_string(root, &queue.queue_file)),
            queue_name: Some(queue.queue_name.clone()),
            job: producer.site.job.clone(),
            message: message(&queue.queue_name, producer, &others, root),
            severity: None,
        })
        .collect()
}

fn ordered<'a>(producers: &[&'a InternalProducer]) -> Vec<&'a InternalProducer> {
    let mut ordered = producers.to_vec();
    ordered.sort_by(|left, right| {
        (
            left.site.file.as_path(),
            left.site.line,
            left.site.job.as_deref(),
        )
            .cmp(&(
                right.site.file.as_path(),
                right.site.line,
                right.site.job.as_deref(),
            ))
    });
    ordered
}

fn message(
    queue: &str,
    producer: &InternalProducer,
    others: &[&InternalProducer],
    root: &Path,
) -> String {
    let (own, rest) = match producer.site.priority {
        EnqueuePriority::Prioritized(value) => {
            (format!("sets priority {value}"), "enqueue with priority 0")
        }
        _ => (
            "enqueues with priority 0".to_string(),
            "set a priority greater than 0",
        ),
    };
    let names = others
        .iter()
        .map(|other| label(root, other))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "queue \"{queue}\" mixes priority 0 and priority > 0; this producer {own} while other producers {rest}: {names}"
    )
}

fn label(root: &Path, producer: &InternalProducer) -> String {
    let place = format!(
        "{}:{}",
        relative_string(root, &producer.site.file),
        producer.site.line
    );
    match (producer.site.job.as_deref(), producer.site.priority) {
        (Some(job), EnqueuePriority::Prioritized(value)) => {
            format!("{place} ({job}, priority {value})")
        }
        (Some(job), _) => format!("{place} ({job})"),
        (None, EnqueuePriority::Prioritized(value)) => format!("{place} (priority {value})"),
        (None, _) => place,
    }
}
