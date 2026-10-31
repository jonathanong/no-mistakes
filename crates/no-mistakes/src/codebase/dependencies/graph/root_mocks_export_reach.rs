type SuppressedCalls = FxHashSet<(PathBuf, u32)>;
type MockGuard = (FxHashSet<NodeId>, SuppressedCalls);

pub(crate) struct MockExportReach {
    ordinary: FxHashSet<NodeId>,
    guards: Vec<MockGuard>,
}

impl DepGraph {
    pub(crate) fn mock_export_reach(
        &self,
        roots: &[NodeId],
        max_depth: Option<usize>,
    ) -> MockExportReach {
        let mut groups: FxHashMap<Option<&Path>, Vec<NodeId>> = fx_map();
        for root in roots {
            groups.entry(root.as_file()).or_default().push(root.clone());
        }
        let mut ordinary = Vec::new();
        let mut guards = Vec::new();
        for (file, group) in groups {
            let Some(boundaries) = file.and_then(|file| self.root_mock_boundaries.get(file)) else {
                ordinary.extend(group);
                continue;
            };
            if boundaries.blocked.is_empty() && boundaries.suppressed.is_empty() {
                ordinary.extend(group);
                ordinary.extend(boundaries.factories.iter().cloned());
                continue;
            }
            let nodes = if boundaries.blocked.is_empty() {
                let mut seeds = group;
                seeds.extend(boundaries.factories.iter().cloned());
                self.call_source_nodes(&seeds, CallTraversal::Transitive, max_depth)
            } else {
                self.walk_with_mock_boundaries(&group, max_depth, boundaries)
            };
            guards.push((nodes, boundaries.suppressed.clone()));
        }
        MockExportReach {
            ordinary: self.call_source_nodes(&ordinary, CallTraversal::Transitive, max_depth),
            guards,
        }
    }
}

pub(crate) fn mock_reach_keeps(
    reach: &MockExportReach,
    source: &NodeId,
    file: &Path,
    offset: u32,
) -> bool {
    if reach.ordinary.contains(source) {
        return true;
    }
    let mut reached = false;
    for (nodes, suppressed) in &reach.guards {
        if !nodes.contains(source) {
            continue;
        }
        reached = true;
        if !suppressed.contains(&(file.to_path_buf(), offset)) {
            return true;
        }
    }
    !reached
}
