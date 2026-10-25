use super::*;

pub(super) fn producer_functions(sources: &Sources<'_>, index: &Index) -> FxHashMap<usize, Taint> {
    let mut functions = sources
        .producers
        .iter()
        .map(|id| (*id, Taint::executor()))
        .collect::<FxHashMap<_, _>>();
    let mut initial_functions: VecDeque<_> = sources.producers.iter().copied().collect();
    while let Some(id) = initial_functions.pop_front() {
        for alias in index.aliases.get(&id).into_iter().flatten() {
            let value = functions[&id].clone();
            if functions.entry(*alias).or_default().join(value) {
                initial_functions.push_back(*alias);
            }
        }
    }
    functions
}

pub(super) fn apply_node_seeds(
    value: &mut Taint,
    at: Span,
    nodes: &FxHashMap<Span, Node>,
    sources: &Sources<'_>,
) {
    if sources.data.contains(&at) || sources.candidates.contains(&at) {
        if let Some(Node::Ref(id, _)) = nodes
            .get(&at)
            .filter(|node| matches!(node, Node::Ref(id, _) if sources.producers.contains(id)))
        {
            value.join(Taint::producer(*id));
        } else {
            value.join(Taint::executor());
        }
    }
    if matches!(nodes.get(&at), Some(Node::Object(_))) {
        value.objects.insert((Vec::new(), at));
    }
}
