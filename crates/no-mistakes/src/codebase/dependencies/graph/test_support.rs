use super::{DepGraph, EdgeKind, EdgeMap, NodeId};
use std::collections::HashMap;
use std::path::PathBuf;

/// Construct a graph directly from pre-built maps for tests.
pub(crate) fn from_raw_maps(
    root: PathBuf,
    forward: HashMap<PathBuf, Vec<PathBuf>>,
    reverse: HashMap<PathBuf, Vec<PathBuf>>,
) -> DepGraph {
    let typed_fwd: EdgeMap = forward
        .into_iter()
        .map(|(k, vs)| {
            (
                NodeId::File(k),
                vs.into_iter()
                    .map(|v| (NodeId::File(v), EdgeKind::Import))
                    .collect(),
            )
        })
        .collect();
    let typed_rev: EdgeMap = reverse
        .into_iter()
        .map(|(k, vs)| {
            (
                NodeId::File(k),
                vs.into_iter()
                    .map(|v| (NodeId::File(v), EdgeKind::Import))
                    .collect(),
            )
        })
        .collect();
    DepGraph {
        root,
        forward: typed_fwd,
        reverse: typed_rev,
    }
}

/// Construct a graph directly from typed edge maps for tests that need non-File nodes.
pub(crate) fn from_typed_maps(root: PathBuf, forward: EdgeMap, reverse: EdgeMap) -> DepGraph {
    DepGraph {
        root,
        forward,
        reverse,
    }
}
