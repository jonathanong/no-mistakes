use super::*;

impl Tracker {
    pub(super) fn project_uses(
        &self,
        facts: &FxHashMap<Span, Taint>,
        sources: &Sources<'_>,
        bindings: &FxHashSet<usize>,
        roots: &mut Vec<Span>,
    ) -> Vec<(Span, bool)> {
        let mut uses = Vec::new();
        for (callee, arguments) in &self.calls {
            let member = uncopied(*callee, &self.nodes);
            let value = facts.get(callee).cloned().unwrap_or_default();
            let factory = value.factory_value()
                && reference_root(*callee, &self.nodes)
                    .is_none_or(|id| !sources.writes.contains(&id));
            if factory {
                roots.push(*callee);
            }
            let receiver = match self.nodes.get(&member) {
                Some(Node::Read(at, key)) => Some((*at, key.as_deref())),
                _ => None,
            };
            let receiver_tainted =
                receiver.is_some_and(|(at, _)| facts.get(&at).is_some_and(Taint::present));
            let executor_metadata = receiver.is_some_and(|(at, key)| {
                facts.get(&at).is_some_and(Taint::executor_leaf)
                    && key.is_some_and(|key| !matches!(key, "query" | "call" | "apply" | "bind"))
            });
            let covered_method = receiver.is_some_and(|(at, key)| {
                key.is_some_and(|key| self.method_checked(at, key, facts, sources))
            });
            if executor_metadata {
                roots.push(*callee);
            } else if !factory
                && !covered_method
                && (value.executable()
                    || (value.latent() && matches!(self.nodes.get(&member), Some(Node::Ref(_, _)))))
            {
                if reference_root(*callee, &self.nodes)
                    .is_none_or(|id| !sources.initial.contains(&id) && !bindings.contains(&id))
                {
                    uses.push((*callee, true));
                }
            } else if receiver_tainted && !covered_method && !factory {
                // Initial receivers keep their existing configured member classification.
                // Newly transferred containers still require opaque method effects.
                if reference_root(*callee, &self.nodes)
                    .is_none_or(|id| !sources.initial.contains(&id))
                {
                    uses.push((receiver.unwrap().0, true));
                }
            } else if covered_method || !receiver_tainted && value.derived && !value.present() {
                roots.push(*callee);
            }
            for argument in arguments {
                if facts.get(argument).is_some_and(Taint::present) {
                    uses.push((*argument, false));
                } else if facts.get(argument).is_some_and(|value| value.derived) {
                    roots.push(*argument);
                }
            }
        }
        uses
    }
}
