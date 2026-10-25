use super::*;
use std::collections::VecDeque;

impl Bindings<'_> {
    pub fn finish(mut self) -> (FxHashSet<usize>, FxHashSet<Span>, FxHashSet<Span>) {
        let (values, method_uses, mut transports) = self.methods.finish(&mut self.factories);
        self.values.extend(values);
        let mut queue: VecDeque<_> = self
            .factories
            .iter()
            .map(|id| (*id, true))
            .chain(self.values.iter().map(|id| (*id, false)))
            .collect();
        while let Some((source, factory)) = queue.pop_front() {
            if !factory {
                transports.extend(self.methods.returned_roots(source));
            }
            for owner in self.methods.returned_owners(source, factory) {
                if self.factories.insert(*owner) {
                    queue.push_back((*owner, true));
                }
            }
            if factory {
                for target in self.calls.get(&source).into_iter().flatten() {
                    if self.values.insert(*target) {
                        queue.push_back((*target, false));
                    }
                }
            }
            for target in self.aliases.get(&source).into_iter().flatten() {
                let inserted = if factory {
                    self.factories.insert(*target)
                } else {
                    self.values.insert(*target)
                };
                if inserted {
                    queue.push_back((*target, factory));
                }
            }
        }
        let mut uses = FxHashSet::default();
        for (id, call, callee) in self.invocations {
            if self.factories.contains(&id) {
                uses.insert(call);
                transports.insert(callee);
            }
        }
        uses.extend(
            self.references
                .into_iter()
                .filter_map(|(id, at)| self.factories.contains(&id).then_some(at)),
        );
        uses.extend(method_uses);
        (self.values, uses, transports)
    }
}
