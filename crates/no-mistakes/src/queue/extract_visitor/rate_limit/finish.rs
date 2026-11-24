use super::{Owner, Processor, QueueVisitor, State, Use, UseKind};
use crate::fx::FxHashMap;
use crate::queue::extract_model::RateLimitSite;
use crate::queue::source::line_number;

impl QueueVisitor<'_> {
    pub(in crate::queue) fn finish_shared_rate_limits(&mut self) {
        let mut state = std::mem::take(&mut self.shared_rate_limit);
        let mut owners: FxHashMap<u32, Vec<Owner>> = FxHashMap::default();
        for (processor, owner) in std::mem::take(&mut state.processors) {
            let start = match processor {
                Processor::Named(symbol) => state.named.get(&symbol).copied(),
                Processor::Inline(start) => Some(start),
            };
            if let Some(start) = start {
                owners.entry(start).or_default().push(owner);
            }
        }
        for site in &state.uses {
            let Some((start, processors)) = site
                .functions
                .iter()
                .rev()
                .find_map(|span| owners.get_key_value(span))
            else {
                continue;
            };
            for owner in processors
                .iter()
                .filter(|owner| state.matches(site, *start, owner))
            {
                self.facts.rate_limits.push(RateLimitSite {
                    line: line_number(self.source, site.start),
                    worker_index: owner.worker,
                });
            }
        }
    }
}

impl State {
    fn matches(&self, site: &Use, start: u32, owner: &Owner) -> bool {
        match &site.kind {
            UseKind::Throw(member) => self.matches_value(member),
            UseKind::Call(symbol) => {
                self.local_receivers.get(&site.start) == Some(&owner.worker)
                    || owner.configured_factory
                        && symbol
                            .and_then(|symbol| self.parameters.get(&symbol))
                            .is_some_and(|(function, ty)| {
                                *function == start && self.matches_type(ty)
                            })
            }
        }
    }
}
