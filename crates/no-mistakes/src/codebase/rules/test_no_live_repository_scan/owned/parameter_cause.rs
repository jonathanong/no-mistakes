//! The parameter-root walk. A failure names the caller or route that stopped it.
use super::cause::{missing_caller, Cause, CauseKind};
use super::parameters::Parameter;
use super::Owners;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::PathOrigin;
use crate::fx::FxHashSet;

impl Owners<'_> {
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
            .ok_or_else(|| step::not_named(site, origin))?;
        let start = source.clone();
        let declared = origin.owner_line.unwrap_or(1);
        let mut pending = vec![(
            source,
            index,
            origin.param_member.clone(),
            declared,
            origin.param_default.clone(),
            origin.relative,
            origin.require_scalar,
        )];
        let mut seen = FxHashSet::default();
        let mut seeded = false;
        let mut forwarded = None;
        let mut blocker = None;
        while let Some((source, index, member, line, default, relative, require_scalar)) =
            pending.pop()
        {
            if !seen.insert((
                source.clone(),
                index,
                member.clone(),
                default.clone(),
                relative,
                require_scalar,
            )) {
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
                if cause.kind != CauseKind::CallersNotProven {
                    return Err(cause);
                }
                blocker.get_or_insert(cause);
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
                let steps = self.caller_steps(
                    caller,
                    Parameter {
                        index,
                        member: member.as_deref(),
                        default: default.as_ref(),
                        relative,
                        require_scalar,
                    },
                    facts,
                    fixture_dirs,
                );
                for step in steps {
                    match step {
                        Step::Block(cause) => return Err(cause),
                        Step::Seed => seeded = true,
                        Step::Follow(
                            next,
                            next_index,
                            next_member,
                            cause,
                            next_line,
                            next_default,
                            next_relative,
                            next_scalar,
                        ) => {
                            forwarded = Some(cause);
                            pending.push((
                                next,
                                next_index,
                                next_member,
                                next_line,
                                next_default,
                                next_relative,
                                next_scalar,
                            ));
                        }
                    }
                }
            }
        }
        if seeded {
            blocker.map_or(Ok(()), Err)
        } else {
            Err(forwarded.unwrap_or_else(|| missing_caller(facts, &start, declared)))
        }
    }
}

#[path = "parameter_step.rs"]
mod parameter_step;
pub(super) use parameter_step::Step;

#[path = "parameter_cause_step.rs"]
pub(super) mod step;

#[cfg(test)]
#[path = "parameter_cause/tests.rs"]
mod tests;

#[path = "parameter_ownership.rs"]
mod ownership;
