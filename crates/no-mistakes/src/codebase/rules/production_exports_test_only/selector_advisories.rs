use crate::codebase::dependencies::graph::{DepGraph, NodeId};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn for_export(
    root: &Path,
    graph: &DepGraph,
    unknown: &BTreeSet<(PathBuf, NodeId)>,
    name: &str,
) -> Option<(PathBuf, usize, String)> {
    let mut evidence = BTreeSet::new();
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
            evidence.insert((
                importer.clone(),
                site.line,
                format!(
                    "{path}:{} `{}` ({})",
                    site.line, site.specifier, site.reason,
                ),
            ));
        }
    }
    // Callers have at least one unknown relationship. Group all its source
    // evidence so one export never expands into one advisory per importer.
    let (file, line, _) = evidence.first()?;
    let (file, line) = (file.clone(), *line);
    let detail = evidence
        .into_iter()
        .map(|(_, _, detail)| detail)
        .collect::<Vec<_>>()
        .join(", ");
    Some((
        file,
        line,
        format!("symbol consumers of `{name}` include an unknown import selector: {detail}"),
    ))
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
