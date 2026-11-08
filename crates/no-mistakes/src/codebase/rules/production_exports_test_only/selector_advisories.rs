use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(super) fn for_sites(
    root: &Path,
    graph: &DepGraph,
    unknown: &BTreeSet<(PathBuf, NodeId)>,
) -> Vec<(PathBuf, usize, String)> {
    let mut evidence = BTreeSet::new();
    let mut reasons = BTreeMap::<_, BTreeSet<_>>::new();
    for (importer, target) in unknown {
        let path = relative_slash_path(root, importer);
        let sites = graph.unknown_selector_sites(importer, target);
        if sites.is_empty() {
            evidence.insert((
                importer.clone(),
                1,
                format!(
                    "{path}: missing prepared selector provenance: \
                     the graph lacks the import line, module specifier, and selector reason; \
                     rebuild analysis with symbol import selector facts and their source evidence, \
                     or replace the import/re-export with explicit names"
                ),
            ));
        }
        for site in sites {
            reasons
                .entry((importer.clone(), site.line, site.specifier.clone()))
                .or_default()
                .insert(site.reason);
        }
    }
    for ((file, line, specifier), reasons) in reasons {
        let path = relative_slash_path(root, &file);
        let reasons = reasons.into_iter().collect::<Vec<_>>().join(", ");
        evidence.insert((
            file,
            line,
            format!("{path}:{line} `{specifier}` ({reasons})"),
        ));
    }
    evidence
        .into_iter()
        .map(|(file, line, detail)| {
            (
                file,
                line,
                format!("production export consumers include an unknown import selector: {detail}"),
            )
        })
        .collect()
}

/// Type-only importers of an export whose value-or-type kind is unknown.
pub(super) fn undetermined(
    root: &Path,
    graph: &DepGraph,
    typed: &BTreeSet<(PathBuf, NodeId)>,
    name: &str,
) -> (PathBuf, usize, String) {
    let mut evidence = BTreeSet::new();
    for (importer, target) in typed {
        let path = relative_slash_path(root, importer);
        let line = graph
            .unknown_selector_sites(importer, target)
            .first()
            .map_or(1, |site| site.line);
        evidence.insert((importer.clone(), line, format!("{path}:{line}")));
    }
    let (file, line, _) = evidence.first().cloned().expect("typed importers exist");
    let sites = evidence
        .into_iter()
        .map(|(_, _, site)| site)
        .collect::<Vec<_>>()
        .join(", ");
    (
        file,
        line,
        format!(
            "type-only references to `{name}` ({sites}) cannot be classified because its export kind \
             (value or type) is undetermined"
        ),
    )
}
