use super::imports::{ImportOrigin, Member, MemberObject, ThrowTarget};
use super::locals::ObjProp;
use super::{Owner, Processor, QueueVisitor, State, Use, UseKind};
use crate::fx::FxHashMap;
use crate::queue::extract_model::{RateLimitSite, RateLimitSiteKind};
use crate::queue::source::line_number;
use oxc_syntax::symbol::SymbolId;

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
            for owner in processors {
                if let Some(kind) = state.classify(site, *start, owner) {
                    self.facts.rate_limits.push(RateLimitSite {
                        line: line_number(self.source, site.start),
                        worker_index: owner.worker,
                        kind,
                    });
                }
            }
        }
    }
}

impl State {
    fn classify(&self, site: &Use, start: u32, owner: &Owner) -> Option<RateLimitSiteKind> {
        match &site.kind {
            UseKind::Throw(target) => self.classify_throw(target),
            UseKind::Call(symbol) => {
                let matched = self.local_receivers.get(&site.start) == Some(&owner.worker)
                    || owner.configured_factory
                        && symbol
                            .and_then(|symbol| self.parameters.get(&symbol))
                            .is_some_and(|(function, ty)| {
                                *function == start && self.matches_type(ty)
                            });
                matched.then_some(RateLimitSiteKind::Confirmed)
            }
        }
    }

    fn classify_throw(&self, target: &ThrowTarget) -> Option<RateLimitSiteKind> {
        match target {
            ThrowTarget::Identifier {
                symbol,
                is_rate_limit_name,
            } => self.classify_identifier(*symbol, *is_rate_limit_name),
            ThrowTarget::MemberRateLimit { object } => self.classify_member(object),
        }
    }

    fn value_import(&self, sym: SymbolId) -> Option<&ImportOrigin> {
        self.imports
            .get(&sym)
            .filter(|import| import.value)
            .map(|import| &import.origin)
    }

    fn classify_identifier(
        &self,
        symbol: Option<SymbolId>,
        is_rate_limit_name: bool,
    ) -> Option<RateLimitSiteKind> {
        let Some(sym) = symbol.map(|sym| self.resolve_alias(sym)) else {
            return is_rate_limit_name.then_some(RateLimitSiteKind::Unresolved);
        };
        if self.local_non_queue.contains(&sym) {
            return None;
        }
        if let Some(origin) = self.value_import(sym) {
            return match origin {
                ImportOrigin::Queue(Member::Error) => Some(RateLimitSiteKind::Confirmed),
                ImportOrigin::LocalNamed { source, imported } => {
                    Some(RateLimitSiteKind::ImportedError {
                        source: source.clone(),
                        imported: imported.clone(),
                        is_rate_limit_name,
                    })
                }
                ImportOrigin::Queue(_) | ImportOrigin::LocalNamespace { .. } => None,
            };
        }
        is_rate_limit_name.then_some(RateLimitSiteKind::Unresolved)
    }

    fn classify_member(&self, object: &MemberObject) -> Option<RateLimitSiteKind> {
        match object {
            MemberObject::Identifier(Some(sym)) => {
                let sym = self.resolve_alias(*sym);
                if let Some(prop) = self.local_object_rate_limit.get(&sym) {
                    return match prop {
                        ObjProp::Absent | ObjProp::NonQueue => None,
                        ObjProp::Symbol(inner) => self.classify_identifier(*inner, true),
                        ObjProp::Opaque => Some(RateLimitSiteKind::Unresolved),
                    };
                }
                if self.local_non_queue.contains(&sym) {
                    return None;
                }
                match self.value_import(sym) {
                    Some(ImportOrigin::Queue(Member::Worker | Member::Namespace)) => {
                        Some(RateLimitSiteKind::Confirmed)
                    }
                    Some(ImportOrigin::Queue(_)) => None,
                    Some(ImportOrigin::LocalNamed { source, imported }) => {
                        Some(RateLimitSiteKind::ImportedMemberObject {
                            source: source.clone(),
                            imported: imported.clone(),
                        })
                    }
                    Some(ImportOrigin::LocalNamespace { source }) => {
                        Some(RateLimitSiteKind::NamespaceError {
                            source: source.clone(),
                        })
                    }
                    None => Some(RateLimitSiteKind::Unresolved),
                }
            }
            MemberObject::NamespaceWorker {
                symbol: Some(sym),
                member,
            } => match self.value_import(self.resolve_alias(*sym)) {
                Some(ImportOrigin::Queue(Member::Namespace)) => Some(RateLimitSiteKind::Confirmed),
                Some(ImportOrigin::Queue(_)) => None,
                Some(ImportOrigin::LocalNamespace { source }) => {
                    Some(RateLimitSiteKind::NamespaceWorkerError {
                        source: source.clone(),
                        member: member.clone(),
                    })
                }
                Some(ImportOrigin::LocalNamed { source, imported }) => {
                    Some(RateLimitSiteKind::ImportedNamespaceWorkerError {
                        source: source.clone(),
                        imported: imported.clone(),
                        member: member.clone(),
                    })
                }
                None => Some(RateLimitSiteKind::Unresolved),
            },
            MemberObject::Identifier(None)
            | MemberObject::NamespaceWorker { symbol: None, .. }
            | MemberObject::Opaque => Some(RateLimitSiteKind::Unresolved),
        }
    }

    fn resolve_alias(&self, mut symbol: SymbolId) -> SymbolId {
        let mut steps = 0;
        while let Some(&next) = self
            .local_aliases
            .get(&symbol)
            .filter(|&&next| next != symbol && steps < 16)
        {
            symbol = next;
            steps += 1;
        }
        symbol
    }
}
