use super::*;
impl Tracker {
    pub(super) fn method_checked(
        &self,
        receiver: Span,
        key: &str,
        facts: &FxHashMap<Span, Taint>,
        sources: &Sources<'_>,
    ) -> bool {
        let Some(value) = facts.get(&receiver).filter(|value| !value.broad) else {
            return false;
        };
        let origins = value
            .objects
            .iter()
            .filter_map(|(path, at)| path.is_empty().then_some(*at))
            .collect::<Vec<_>>();
        if origins.is_empty() {
            return false;
        }
        origins.into_iter().all(|origin| {
            let Some(scope) = self.methods.get(&(origin, key.to_owned())) else {
                return false;
            };
            if self.returned_scopes.contains(scope) {
                return false;
            }
            self.this_reads
                .get(scope)
                .into_iter()
                .flatten()
                .all(|this| {
                    let Some(reads) = self.member_reads.get(this) else {
                        return false;
                    };
                    !reads.is_empty()
                        && reads.iter().all(|at| match self.nodes.get(at) {
                            Some(Node::Read(_, Some(key))) => {
                                !value.clone().read(Some(key)).present()
                                    || sources.data.contains(at)
                            }
                            _ => false,
                        })
                })
        })
    }
}
