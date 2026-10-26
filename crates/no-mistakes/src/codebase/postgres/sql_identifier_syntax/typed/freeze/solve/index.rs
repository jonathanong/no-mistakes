use super::*;

type BindingProjection = (usize, Vec<Option<String>>);
#[derive(Default)]
pub(super) struct Index {
    pub parents: FxHashMap<Span, Vec<Span>>,
    pub bindings: FxHashMap<usize, Vec<Span>>,
    pub invokes: FxHashMap<usize, std::collections::BTreeSet<Span>>,
    pub outputs: FxHashMap<Span, Vec<BindingProjection>>,
    pub returns: FxHashMap<Span, Vec<usize>>,
    pub aliases: FxHashMap<usize, Vec<usize>>,
    pub stores: FxHashMap<Span, Vec<usize>>,
}
impl Index {
    pub fn new(tracker: &Tracker) -> Self {
        let mut index = Index::default();
        for (at, node) in &tracker.nodes {
            for child in node.children() {
                index.parents.entry(child).or_default().push(*at);
            }
            match node {
                Node::Ref(id, _) => index.bindings.entry(*id).or_default().push(*at),
                Node::Invoke(id) => {
                    index.invokes.entry(*id).or_default().insert(*at);
                }
                // Call dependencies must not become transport-traversal children.
                Node::InvokeValue(callee) => index.parents.entry(*callee).or_default().push(*at),
                _ => {}
            }
        }
        for (id, source, path) in &tracker.flows {
            index
                .outputs
                .entry(*source)
                .or_default()
                .push((*id, path.clone()));
        }
        for (id, source) in &tracker.returns {
            index.returns.entry(*source).or_default().push(*id);
        }
        for (source, target) in &tracker.aliases {
            index.aliases.entry(*source).or_default().push(*target);
        }
        let mut neighbors = FxHashMap::<usize, Vec<usize>>::default();
        for (a, b) in &tracker.aliases {
            neighbors.entry(*a).or_default().push(*b);
            neighbors.entry(*b).or_default().push(*a);
        }
        let mut groups = FxHashMap::<usize, std::sync::Arc<Vec<usize>>>::default();
        for root in neighbors.keys() {
            if groups.contains_key(root) {
                continue;
            }
            let mut related = FxHashSet::from_iter([*root]);
            let mut pending = vec![*root];
            while let Some(id) = pending.pop() {
                for next in neighbors.get(&id).into_iter().flatten() {
                    if related.insert(*next) {
                        pending.push(*next);
                    }
                }
            }
            let group = std::sync::Arc::new(related.into_iter().collect::<Vec<_>>());
            for id in group.iter() {
                groups.insert(*id, group.clone());
            }
        }
        for (target, source) in &tracker.stores {
            let related = groups
                .get(target)
                .map_or_else(|| vec![*target], |ids| ids.as_ref().clone());
            index.stores.entry(*source).or_default().extend(related);
        }
        index
    }
}
