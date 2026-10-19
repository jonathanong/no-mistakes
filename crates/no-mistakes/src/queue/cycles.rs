use crate::queue::graph_model::{CheckFinding, InternalProducer, InternalWorker};
use crate::queue::source::relative_string;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

pub(super) fn queue_cycles(
    root: &Path,
    producers: &[InternalProducer],
    workers: &[InternalWorker],
) -> Vec<CheckFinding> {
    let mut workers_by_file: HashMap<&Path, Vec<&InternalWorker>> = HashMap::new();
    for worker in workers {
        workers_by_file
            .entry(worker.site.file.as_path())
            .or_default()
            .push(worker);
    }
    let mut outgoing: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut origin: HashMap<String, (PathBuf, usize)> = HashMap::new();
    for producer in producers {
        let Some(target) = &producer.queue else {
            continue;
        };
        let Some(handlers) = workers_by_file.get(producer.site.file.as_path()) else {
            continue;
        };
        for worker in handlers {
            let Some(source) = &worker.queue else {
                continue;
            };
            if source.queue_name == target.queue_name {
                continue;
            }
            outgoing
                .entry(source.queue_name.clone())
                .or_default()
                .insert(target.queue_name.clone());
            origin
                .entry(source.queue_name.clone())
                .or_insert_with(|| (worker.site.file.clone(), worker.site.line));
        }
    }
    let mut seen = BTreeSet::new();
    let mut findings = Vec::new();
    for start in outgoing.keys() {
        let mut path = Vec::new();
        walk(start, &outgoing, &mut path, &mut seen, &mut findings);
    }
    findings.sort();
    findings.dedup();
    findings
        .into_iter()
        .map(|cycle| {
            let name = cycle.split(" -> ").next().unwrap_or("");
            let (file, line) = origin
                .get(name)
                .cloned()
                .unwrap_or_else(|| (root.to_path_buf(), 1));
            CheckFinding {
                kind: "queue-cycle".to_string(),
                file: relative_string(root, &file),
                line,
                queue_file: None,
                queue_name: Some(name.to_string()),
                job: None,
                message: format!(
                    "queue cycle {cycle} has no declared terminator; review it before shipping"
                ),
            }
        })
        .collect()
}

fn walk(
    node: &str,
    outgoing: &BTreeMap<String, BTreeSet<String>>,
    path: &mut Vec<String>,
    seen: &mut BTreeSet<String>,
    findings: &mut Vec<String>,
) {
    if let Some(index) = path.iter().position(|name| name == node) {
        let mut cycle = path[index..].to_vec();
        cycle.push(node.to_string());
        let key = canonical(&cycle);
        if seen.insert(key.clone()) {
            findings.push(key);
        }
        return;
    }
    path.push(node.to_string());
    if let Some(nexts) = outgoing.get(node) {
        for next in nexts {
            walk(next, outgoing, path, seen, findings);
        }
    }
    path.pop();
}

fn canonical(cycle: &[String]) -> String {
    let body = &cycle[..cycle.len().saturating_sub(1)];
    let Some(start) = body.iter().min() else {
        return cycle.join(" -> ");
    };
    let offset = body.iter().position(|name| name == start).unwrap_or(0);
    let mut rotated = body[offset..].to_vec();
    rotated.extend(body[..offset].iter().cloned());
    rotated.push(start.clone());
    rotated.join(" -> ")
}
