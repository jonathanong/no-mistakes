impl ResolvedCallAlternatives {
    pub fn known(&self) -> &[ResolvedCallCandidate] {
        &self.known
    }
    /// Normalize once before sharing a set across readers. Different selectors
    /// may name the same callable; its incoming invocation still occurs once.
    pub fn new(mut known: Vec<ResolvedCallCandidate>, has_unknown: bool) -> Self {
        known.sort_unstable();
        known.dedup();
        let mut nodes = known
            .iter()
            .filter_map(|candidate| candidate.node.clone())
            .collect::<Vec<_>>();
        nodes.sort_unstable();
        nodes.dedup();
        Self {
            known,
            has_unknown,
            nodes,
        }
    }
}

/// Borrowed, deduplicated canonical callees of one physical call occurrence.
pub struct ResolvedCallNodes<'a> {
    single: Option<&'a NodeId>,
    alternatives: std::slice::Iter<'a, NodeId>,
}
impl<'a> Iterator for ResolvedCallNodes<'a> {
    type Item = &'a NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.single.take().or_else(|| self.alternatives.next())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = usize::from(self.single.is_some()) + self.alternatives.len();
        (count, Some(count))
    }
}
impl ExactSizeIterator for ResolvedCallNodes<'_> {}

impl ResolvedCallSite {
    pub fn target_nodes(&self) -> ResolvedCallNodes<'_> {
        match &self.target {
            ResolvedCallTarget::Alternatives(alternatives) => ResolvedCallNodes {
                single: None,
                alternatives: alternatives.nodes.iter(),
            },
            _ => ResolvedCallNodes {
                single: self.target_node.as_ref(),
                alternatives: [].iter(),
            },
        }
    }
}
