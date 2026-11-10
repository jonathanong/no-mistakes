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
    if let Some((file, line)) = at {
        return Cause::new(kind, file.clone(), *line, "");
    }
    let (file, line) = function_site(facts, source);
    Cause::new(kind, file, line, "")
}

fn local_route(owners: &Owners<'_>, source: &NodeId, facts: &dyn TsFactLookup) -> Cause {
    let NodeId::Symbol { file, symbol, .. } = source else {
        let (path, line) = function_site(facts, source);
        return Cause::new(CauseKind::NotNamedFunction, path, line, "");
    };
    if owners.escaped.contains(file, symbol) {
        if let Some((path, line, name)) = owners.escaped.reference(file, symbol) {
            return Cause::new(CauseKind::NonDirectReference, path, line, name);
        }
        let (path, line) = function_site(facts, source);
        return Cause::new(
            CauseKind::NonDirectReference,
            path,
            line,
            symbol.to_string(),
        );
    }
    if owners.consumed.contains(file) {
        if let Some(hit) = owners.consumed.route(file) {
            return Cause::new(
                kind_of(hit.kind),
                hit.file.clone(),
                hit.line,
                hit.detail.clone(),
            );
        }
    }
    let (path, line) = function_site(facts, source);
    Cause::new(CauseKind::UnreadModule, path, line, "")
}
