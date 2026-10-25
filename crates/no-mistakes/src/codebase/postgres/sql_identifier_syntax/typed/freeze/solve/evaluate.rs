use super::*;
pub(super) fn evaluate(
    node: &Node,
    facts: &FxHashMap<Span, Taint>,
    values: &FxHashMap<usize, Taint>,
    functions: &FxHashMap<usize, Taint>,
) -> Taint {
    let source = |at| facts.get(&at).cloned().unwrap_or_default();
    match node {
        Node::Scalar | Node::This(_) => Taint::default(),
        Node::Data(values) => Taint {
            derived: values.iter().any(|at| source(*at).derived),
            ..Taint::default()
        },
        Node::Ref(id, _) => {
            let mut value = values.get(id).cloned().unwrap_or_default();
            if functions.get(id).is_some_and(Taint::present) {
                value.join(Taint::producer(*id));
            }
            value
        }
        Node::Invoke(id) => functions.get(id).cloned().unwrap_or_default(),
        Node::Copy(at) => source(*at),
        Node::Read(at, key) => source(*at).read(key.as_deref()),
        Node::Freeze(at) => {
            let mut value = source(*at);
            value.derived = true;
            value
        }
        Node::Union(values) => {
            let mut value = Taint::default();
            for at in values {
                value.join(source(*at));
            }
            value
        }
        Node::Object(fields) => {
            let mut value = Taint::default();
            for (key, at) in fields {
                value.join(match key {
                    Field::Key(key) => source(*at).prefix(key),
                    Field::Spread => source(*at).spread(),
                    Field::Unknown => source(*at).read(None),
                });
            }
            value
        }
    }
}
