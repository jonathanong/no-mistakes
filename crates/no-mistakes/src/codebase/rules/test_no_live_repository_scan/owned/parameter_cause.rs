//! The parameter-root walk. A failure names the caller or route that stopped it.
use super::cause::{missing_caller, Cause, CauseKind, Ownership};
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin};
use crate::fx::FxHashSet;

impl Owners<'_> {
    pub(in super::super) fn ownership(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Ownership {
        let climbs = origin.segments.iter().any(|segment| segment == "..")
            && (origin.param.is_some()
                || origin.env.is_some()
                || origin.anchor == PathAnchor::Temp);
        if climbs {
            if origin.param.is_some() && origin.env.is_none() {
                return Ownership::Parameter(Cause::new(
                    CauseKind::ClimbsOut,
                    site.file.clone(),
                    site.line,
                    "",
                ));
            }
            return Ownership::No;
        }
        match (&origin.env, origin.param) {
            (Some(name), _) => {
                if self.env_owned(name, site, facts) {
                    Ownership::Yes
                } else {
                    Ownership::No
                }
            }
            (None, Some(index)) => match self.param_cause(index, origin, site, facts, fixture_dirs)
            {
                Ok(()) => Ownership::Yes,
                Err(cause) => Ownership::Parameter(cause),
            },
            (None, None) => origin.reassigned_at.map_or(Ownership::No, |line| {
                Ownership::Parameter(Cause::new(
                    CauseKind::ReassignedParameter,
                    site.file.clone(),
                    line,
                    "",
                ))
            }),
        }
    }

    fn param_cause(
        &self,
        index: usize,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Result<(), Cause> {
        let source = self
            .parameter_source(origin, site)
            .ok_or_else(|| not_named(site, origin))?;
        let start = source.clone();
        let declared = origin.owner_line.unwrap_or(1);
        let mut pending = vec![(source, index, origin.param_member.clone(), declared)];
        let mut seen = FxHashSet::default();
        let mut seeded = false;
        let mut forwarded = None;
        while let Some((source, index, member, line)) = pending.pop() {
            if !seen.insert((source.clone(), index, member.clone())) {
                continue;
            }
            let Some(module) = source.as_file() else {
                return Err(self.route_cause(&source, facts));
            };
            if self.closure_unknown(module) {
                return Err(self.route_for(&source, facts, Some(index)));
            }
            if !self.parameter_proven(&source) {
                let routed = self.route_for(&source, facts, Some(index));
                if routed.kind != CauseKind::CallersNotProven {
                    return Err(routed);
                }
            }
            if let Some(cause) = self.caller_blocked(&source, facts) {
                return Err(cause);
            }
            let Some(callers) = self
                .callers
                .get(&source)
                .filter(|callers| !callers.is_empty())
            else {
                return Err(forwarded
                    .clone()
                    .unwrap_or_else(|| missing_caller(facts, &source, line)));
            };
            for caller in callers.iter().copied() {
                let step = self.caller_step(caller, index, member.as_deref(), facts, fixture_dirs);
                match step {
                    Step::Block(cause) => return Err(cause),
                    Step::Seed => seeded = true,
                    Step::Follow(next, next_index, next_member, cause, next_line) => {
                        forwarded = Some(cause);
                        pending.push((next, next_index, next_member, next_line));
                    }
                }
            }
        }
        if seeded {
            Ok(())
        } else {
            Err(forwarded.unwrap_or_else(|| missing_caller(facts, &start, declared)))
        }
    }
}

enum Step {
    Block(Cause),
    Seed,
    Follow(NodeId, usize, Option<String>, Cause, u32),
}

#[path = "parameter_cause_step.rs"]
mod step;

fn not_named(site: &ResolvedCallSite, origin: &PathOrigin) -> Cause {
    let line = origin.owner_line.unwrap_or(1);
    Cause::new(CauseKind::NotNamedFunction, site.file.clone(), line, "")
}

#[cfg(test)]
#[path = "parameter_cause/tests.rs"]
mod tests;
