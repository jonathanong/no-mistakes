impl DepGraph {
    fn automock_initializer_seeds(
        &self,
        target: &Path,
        facts: &dyn TsFactLookup,
    ) -> (bool, Vec<NodeId>) {
        let mut safe_to_block = true;
        let mut seen = fx_set();
        let mut queue = VecDeque::from([NodeId::file(target)]);
        let mut seeds = fx_set();
        let edges = self.edges.forward();
        while let Some(node) = queue.pop_front() {
            let Some(path) = node.as_file() else {
                safe_to_block = false;
                continue;
            };
            if !seen.insert(path.to_path_buf()) {
                continue;
            }
            let Some(file) = facts.get_ts_facts(path) else {
                safe_to_block = false;
                continue;
            };
            if file.parse_error.is_some()
                || file.operational_error.is_some()
                || file.module_bindings.is_none()
            {
                safe_to_block = false;
            }
            if self
                .call_sites_in_file(path)
                .iter()
                .any(|site| site.caller.is_none())
            {
                safe_to_block = false;
                seeds.insert(NodeId::file(path));
            }
            let Some(neighbors) = edges.get(&node) else {
                continue;
            };
            for (neighbor, kind) in &neighbors.neighbors {
                if matches!(
                    kind,
                    EdgeKind::Import
                        | EdgeKind::WorkspaceImport
                        | EdgeKind::Require
                        | EdgeKind::DynamicImport
                        | EdgeKind::ConditionalDynamicImport
                ) {
                    queue.push_back(neighbor.clone());
                }
            }
        }
        let mut seeds = seeds.into_iter().collect::<Vec<_>>();
        seeds.sort();
        (safe_to_block, seeds)
    }
}
