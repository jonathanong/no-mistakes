use super::plan::{impact_reason_label, path_confidence, relative_path, slash_node_name};
use super::{Confidence, ImpactReason, SelectedTest, Warning};
use no_mistakes::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use no_mistakes::config::v2::schema::TestPlanGroupType;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Extra coverage signals derived from a unified diff that the BFS over the
/// dep graph cannot recover on its own. Currently carries identifiers that
/// the diff removed (e.g. `data-pw` selector values) along with a reverse
/// index from each identifier to the test files that still reference it.
///
/// Empty when no diff body is available, which is the case for
/// `--changed-file`-only invocations.
#[derive(Default)]
pub(super) struct CoverageHints {
    pub removed_selectors: BTreeMap<PathBuf, Vec<(String, String)>>,
    pub selector_dependents: HashMap<(String, String), Vec<PathBuf>>,
}

impl CoverageHints {
    fn is_empty(&self) -> bool {
        self.removed_selectors.is_empty()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn group_candidates(
    group: TestPlanGroupType,
    root: &Path,
    changed_files: &[PathBuf],
    graph: &DepGraph,
    all_tests: &[PathBuf],
    all_test_set: &HashSet<PathBuf>,
    used: &HashSet<String>,
    hints: &CoverageHints,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<(String, String)>,
) -> Vec<SelectedTest> {
    match group {
        TestPlanGroupType::Direct => direct_candidates(root, changed_files, all_test_set, used),
        TestPlanGroupType::Coverage | TestPlanGroupType::Dependencies => graph_candidates(
            group,
            root,
            changed_files,
            graph,
            all_test_set,
            used,
            hints,
            warnings,
            warnings_seen,
        ),
        TestPlanGroupType::Sample => sample_candidates(root, all_tests, used),
    }
}

fn direct_candidates(
    root: &Path,
    changed_files: &[PathBuf],
    all_test_set: &HashSet<PathBuf>,
    used: &HashSet<String>,
) -> Vec<SelectedTest> {
    changed_files
        .iter()
        .filter(|changed| all_test_set.contains(*changed))
        .filter_map(|changed| {
            let rel = relative_path(root, changed);
            (!used.contains(&rel)).then(|| SelectedTest {
                test_file: rel.clone(),
                confidence: Confidence::High,
                reasons: vec![ImpactReason {
                    changed_file: rel.clone(),
                    path: vec![rel],
                    via: vec!["self".to_string()],
                }],
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn graph_candidates(
    group: TestPlanGroupType,
    root: &Path,
    changed_files: &[PathBuf],
    graph: &DepGraph,
    all_test_set: &HashSet<PathBuf>,
    used: &HashSet<String>,
    hints: &CoverageHints,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<(String, String)>,
) -> Vec<SelectedTest> {
    let mut selected: BTreeMap<String, SelectedTest> = BTreeMap::new();
    for changed in changed_files {
        if all_test_set.contains(changed) {
            continue;
        }
        let rel_changed = relative_path(root, changed);
        let start_node = NodeId::File(changed.clone());
        let (reachable_tests, path_parents) = bfs_path_find_set(graph, &start_node, all_test_set);
        for (test_node, edge_path) in reachable_tests {
            let NodeId::File(test_path) = &test_node else {
                continue;
            };
            let is_coverage = edge_path.iter().any(|edge| {
                matches!(
                    edge,
                    EdgeKind::RouteTest | EdgeKind::Layout | EdgeKind::Selector
                )
            });
            if (group == TestPlanGroupType::Coverage) != is_coverage {
                continue;
            }
            let rel_test = relative_path(root, test_path);
            if used.contains(&rel_test) {
                continue;
            }
            let reason = reason_from_path(
                root,
                &rel_changed,
                &test_node,
                &path_parents,
                &edge_path,
                warnings,
                warnings_seen,
            );
            let entry = selected
                .entry(rel_test.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test,
                    confidence: path_confidence(&edge_path),
                    reasons: Vec::new(),
                });
            let confidence = path_confidence(&edge_path);
            if confidence > entry.confidence {
                entry.confidence = confidence;
            }
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
        }
    }
    if group == TestPlanGroupType::Coverage && !hints.is_empty() {
        append_removed_selector_candidates(root, all_test_set, used, hints, &mut selected);
    }
    selected.into_values().collect()
}

fn append_removed_selector_candidates(
    root: &Path,
    all_test_set: &HashSet<PathBuf>,
    used: &HashSet<String>,
    hints: &CoverageHints,
    selected: &mut BTreeMap<String, SelectedTest>,
) {
    let confidence = path_confidence(&[EdgeKind::Selector]);
    // Iterate the hint map directly so deleted source files (which are
    // filtered out of `changed_files` because they no longer exist on disk)
    // still contribute their removed identifiers to the coverage hints.
    for (changed_path, removed) in &hints.removed_selectors {
        let rel_changed = relative_path(root, changed_path);
        let mut tests_for_changed: HashSet<PathBuf> = HashSet::new();
        for pair in removed {
            let Some(tests) = hints.selector_dependents.get(pair) else {
                continue;
            };
            for test in tests {
                if test == changed_path {
                    continue;
                }
                if !all_test_set.contains(test) {
                    continue;
                }
                tests_for_changed.insert(test.clone());
            }
        }
        let mut sorted: Vec<PathBuf> = tests_for_changed.into_iter().collect();
        sorted.sort();
        for test_path in sorted {
            let rel_test = relative_path(root, &test_path);
            if used.contains(&rel_test) {
                continue;
            }
            let reason = ImpactReason {
                changed_file: rel_changed.clone(),
                path: vec![rel_changed.clone(), rel_test.clone()],
                via: vec!["selector".to_string()],
            };
            let entry = selected
                .entry(rel_test.clone())
                .or_insert_with(|| SelectedTest {
                    test_file: rel_test,
                    confidence,
                    reasons: Vec::new(),
                });
            if confidence > entry.confidence {
                entry.confidence = confidence;
            }
            if !entry.reasons.contains(&reason) {
                entry.reasons.push(reason);
            }
        }
    }
}

fn sample_candidates(
    root: &Path,
    all_tests: &[PathBuf],
    used: &HashSet<String>,
) -> Vec<SelectedTest> {
    all_tests
        .iter()
        .filter_map(|test| {
            let rel = relative_path(root, test);
            (!used.contains(&rel)).then(|| SelectedTest {
                test_file: rel.clone(),
                confidence: Confidence::Low,
                reasons: vec![ImpactReason {
                    changed_file: "*sample*".to_string(),
                    path: vec![rel],
                    via: vec!["sample".to_string()],
                }],
            })
        })
        .collect()
}

pub(super) fn stable_take(mut candidates: Vec<SelectedTest>, limit: usize) -> Vec<SelectedTest> {
    candidates.sort_by(|a, b| stable_test_key(&a.test_file).cmp(&stable_test_key(&b.test_file)));
    candidates.truncate(limit);
    candidates.sort_by(|a, b| a.test_file.cmp(&b.test_file));
    candidates
}

fn stable_test_key(path: &str) -> (u64, &str) {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for byte in path.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    (hash, path)
}

pub(super) fn selected_from_paths(
    root: &Path,
    tests: &[PathBuf],
    via: &str,
    changed_file: Option<&PathBuf>,
) -> Vec<SelectedTest> {
    let changed = changed_file
        .map(|path| relative_path(root, path))
        .unwrap_or_else(|| "*all*".to_string());
    tests
        .iter()
        .map(|test| {
            let rel = relative_path(root, test);
            SelectedTest {
                test_file: rel.clone(),
                confidence: Confidence::High,
                reasons: vec![ImpactReason {
                    changed_file: changed.clone(),
                    path: vec![changed.clone(), rel],
                    via: vec![via.to_string()],
                }],
            }
        })
        .collect()
}

pub(super) fn merge_selected(existing: &mut SelectedTest, next: &SelectedTest) {
    if next.confidence > existing.confidence {
        existing.confidence = next.confidence;
    }
    for reason in &next.reasons {
        if !existing.reasons.contains(reason) {
            existing.reasons.push(reason.clone());
        }
    }
}

fn reason_from_path(
    root: &Path,
    rel_changed: &str,
    test_node: &NodeId,
    path_parents: &HashMap<NodeId, (NodeId, EdgeKind)>,
    edge_path: &[EdgeKind],
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<(String, String)>,
) -> ImpactReason {
    let mut node_chain = Vec::new();
    let mut curr = test_node.clone();
    node_chain.push(slash_node_name(&curr, root));

    while let Some((parent, kind)) = path_parents.get(&curr) {
        node_chain.push(slash_node_name(parent, root));
        push_edge_warning(root, &curr, parent, *kind, warnings, warnings_seen);
        curr = parent.clone();
    }
    node_chain.reverse();

    ImpactReason {
        changed_file: rel_changed.to_string(),
        path: node_chain,
        via: edge_path
            .iter()
            .map(|kind| impact_reason_label(*kind).to_string())
            .collect(),
    }
}

fn push_edge_warning(
    root: &Path,
    curr: &NodeId,
    parent: &NodeId,
    kind: EdgeKind,
    warnings: &mut Vec<Warning>,
    warnings_seen: &mut HashSet<(String, String)>,
) {
    let (r#type, message, file) = match kind {
        EdgeKind::DynamicImport => {
            let file = slash_node_name(curr, root);
            (
                "dynamic-import",
                format!("Dynamic import in `{}` might not be fully resolved.", file),
                file,
            )
        }
        EdgeKind::HttpCall => {
            let file = slash_node_name(curr, root);
            (
                "http-call",
                format!(
                    "Dynamic HTTP call in `{}` to backend `{}`.",
                    file,
                    slash_node_name(parent, root)
                ),
                file,
            )
        }
        EdgeKind::ProcessSpawn => {
            let file = slash_node_name(curr, root);
            (
                "process-spawn",
                format!("Process spawned in `{}`.", file),
                file,
            )
        }
        _ => return,
    };
    let warn = Warning {
        r#type: r#type.to_string(),
        message,
        file,
    };
    if warnings_seen.insert((warn.r#type.clone(), warn.file.clone())) {
        warnings.push(warn);
    }
}

#[allow(clippy::type_complexity)]
fn bfs_path_find_set(
    graph: &DepGraph,
    start: &NodeId,
    test_files: &HashSet<PathBuf>,
) -> (
    Vec<(NodeId, Vec<EdgeKind>)>,
    HashMap<NodeId, (NodeId, EdgeKind)>,
) {
    let mut queue = VecDeque::new();
    let mut parents: HashMap<NodeId, (NodeId, EdgeKind)> = HashMap::new();
    let mut visited = HashSet::new();
    let mut reachable = Vec::new();

    queue.push_back(start.clone());
    visited.insert(start.clone());

    while let Some(current) = queue.pop_front() {
        if let NodeId::File(path) = &current {
            if current != *start && test_files.contains(path) {
                let mut edge_path = Vec::new();
                let mut curr_node = current.clone();
                while let Some((parent, kind)) = parents.get(&curr_node) {
                    edge_path.push(*kind);
                    curr_node = parent.clone();
                }
                edge_path.reverse();
                reachable.push((current.clone(), edge_path));
            }
        }

        if let Some(neighbors) = graph.dependents_of_node(&current) {
            for (neighbor, kind) in neighbors {
                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    parents.insert(neighbor.clone(), (current.clone(), *kind));
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    (reachable, parents)
}
