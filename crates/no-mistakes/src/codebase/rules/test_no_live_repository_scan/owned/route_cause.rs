//! The route that made a parameter's callers unknowable.
use super::cause::{function_site, kind_of, Cause, CauseKind};
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, TsFactLookup};

impl Owners<'_> {
    pub(super) fn route_cause(&self, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
        if !self.setups.known() {
            return hole(
                CauseKind::UnresolvedSetup,
                self.setup_hole.as_ref(),
                source,
                facts,
            );
        }
        if self.escaped.unknown() {
            return hole(
                CauseKind::UnreadModule,
                self.escaped.unknown_at(),
                source,
                facts,
            );
        }
        if self.consumed.unknown() {
            return hole(
                CauseKind::UnreadModule,
                self.consumed.unknown_at(),
                source,
                facts,
            );
        }
        local_route(self, source, facts)
    }
}

fn hole(
    kind: CauseKind,
    at: Option<&(std::path::PathBuf, u32)>,
    source: &NodeId,
    facts: &dyn TsFactLookup,
) -> Cause {
    let (file, line) = at.cloned().unwrap_or_else(|| function_site(facts, source));
    Cause::new(kind, file, line, "")
}

fn local_route(owners: &Owners<'_>, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
    let NodeId::Symbol { file, symbol, .. } = source else {
        return hole(CauseKind::NotNamedFunction, None, source, facts);
    };
    if owners.escaped.contains(file, symbol) {
        let found = owners.escaped.reference(file, symbol);
        let (path, line, name) = found.unwrap_or_else(|| named_site(facts, source, symbol));
        return Cause::new(CauseKind::NonDirectReference, path, line, name);
    }
    let route = owners
        .consumed
        .route(file)
        .filter(|_| owners.consumed.contains(file));
    match route {
        Some(hit) => Cause::new(
            kind_of(hit.kind),
            hit.file.clone(),
            hit.line,
            hit.detail.clone(),
        ),
        None => hole(CauseKind::UnreadModule, None, source, facts),
    }
}

fn named_site(
    facts: &dyn TsFactLookup,
    source: &NodeId,
    name: &str,
) -> (std::path::PathBuf, u32, String) {
    let (path, line) = function_site(facts, source);
    (path, line, name.to_string())
}
