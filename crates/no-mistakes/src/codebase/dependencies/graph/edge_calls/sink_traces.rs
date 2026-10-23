impl DepGraph {
    /// Traverse canonical calls in FIFO order, materializing only requested sink paths.
    /// The root itself remains the consumer's responsibility, as in `call_traces`.
    pub(crate) fn call_traces_to_sinks_excluding(
        &self,
        root: &NodeId,
        sinks: &FxHashSet<&NodeId>,
        excluded: &FxHashSet<&NodeId>,
    ) -> Vec<CallTrace> {
        let edges = self.traversal_edges().forward();
        let mut out = Vec::new();
        for root in normalize_nodes(&self.resolve_display_callable(root)) {
            if excluded.contains(&root) {
                continue;
            }
            let mut seen = fx_set();
            // Borrow graph identities; indices retain first-predecessor ties without
            // cloning an entire prefix at every intermediate call.
            let mut queue = vec![(&root, None::<usize>)];
            seen.insert(&root);
            let mut cursor = 0;
            while cursor < queue.len() {
                let node = queue[cursor].0;
                if let Some(neighbors) = edges.get(node) {
                    for (neighbor, kind) in &neighbors.neighbors {
                        if *kind != EdgeKind::Call
                            || excluded.contains(neighbor)
                            || !seen.insert(neighbor)
                        {
                            continue;
                        }
                        let index = queue.len();
                        queue.push((neighbor, Some(cursor)));
                        if sinks.contains(neighbor) {
                            let mut nodes = Vec::new();
                            let mut predecessor = Some(index);
                            while let Some(index) = predecessor {
                                nodes.push(queue[index].0.clone());
                                predecessor = queue[index].1;
                            }
                            nodes.reverse();
                            #[cfg(test)]
                            CALL_PATH_COUNT.with(|count| count.set(count.get() + 1));
                            out.push(CallTrace {
                                root: root.clone(),
                                target: neighbor.clone(),
                                nodes,
                            });
                        }
                    }
                }
                cursor += 1;
            }
        }
        out
    }
}

#[cfg(test)]
thread_local! {
    static CALL_PATH_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn take_call_path_count() -> usize {
    CALL_PATH_COUNT.with(|count| count.replace(0))
}
