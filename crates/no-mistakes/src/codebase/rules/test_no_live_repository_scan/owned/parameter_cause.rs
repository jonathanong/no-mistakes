//! The parameter-root walk. A failure names the caller or route that stopped it.
use super::cause::{missing_caller, Cause, CauseKind, Ownership};
use super::parameters::{argument_path, argument_present, argument_text, unproven_argument};
use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::test_no_live_repository_scan::provenance::{
    arg_block, classify, live_or_variable, missing_block, ArgBlock, Verdict,
};
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathOrigin, PathProvenance};
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
            if self.closure_unknown(module) || !self.parameter_proven(&source) {
                return Err(self.route_cause(&source, facts));
            }
            if let Some(cause) = self.caller_blocked(&source, facts) {
                return Err(cause);
            }
            let Some(callers) = self
                .callers
                .get(&source)
                .filter(|callers| !callers.is_empty())
            else {
                return Err(missing_caller(facts, &source, line));
            };
            for caller in callers.iter().copied() {
                let step = self.caller_step(caller, index, member.as_deref(), facts, fixture_dirs);
                match step {
                    Step::Block(cause) => return Err(cause),
                    Step::Seed => seeded = true,
                    Step::Follow(next, next_index, next_member, cause, next_line) => {
                        if forwarded.is_none() {
                            forwarded = Some(cause);
                        }
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

impl Owners<'_> {
    fn caller_step(
        &self,
        caller: &ResolvedCallSite,
        index: usize,
        member: Option<&str>,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Step {
        let empty = PathProvenance::default();
        let provenance = facts
            .get_ts_facts(&caller.file)
            .and_then(path_provenance)
            .unwrap_or(&empty);
        let argument = argument_path(provenance, caller.offset, index, member);
        let present = argument_present(provenance, caller.offset, index);
        let text = argument_text(provenance, caller.offset, index);
        if argument.is_none() {
            if let Some(observed) = unproven_argument(provenance, caller.offset, index) {
                if matches!(classify(Some(observed), fixture_dirs), Verdict::Live(_)) {
                    return Step::Block(caller_cause(caller, ArgBlock::Live, text));
                }
            }
        }
        let Some(argument) = argument else {
            return Step::Block(caller_cause(caller, missing_block(present), text));
        };
        if let Some(block) = arg_block(argument, fixture_dirs) {
            return Step::Block(caller_cause(caller, block, text));
        }
        if classify(Some(argument), fixture_dirs) == Verdict::Allowed {
            return Step::Seed;
        }
        if let Some(name) = argument.env.as_deref() {
            if !self.env_argument_owned(name, caller, facts) {
                let block = live_or_variable(argument, fixture_dirs);
                return Step::Block(caller_cause(caller, block, text));
            }
            return Step::Seed;
        }
        let source = argument.param.zip(self.parameter_source(argument, caller));
        let Some((next_index, next)) = source else {
            return Step::Block(caller_cause(caller, ArgBlock::Forwarded, text));
        };
        Step::Follow(
            next,
            next_index,
            argument.param_member.clone(),
            caller_cause(caller, ArgBlock::Forwarded, text),
            argument.owner_line.unwrap_or(caller.line),
        )
    }
}

fn caller_cause(caller: &ResolvedCallSite, block: ArgBlock, text: String) -> Cause {
    let kind = match block {
        ArgBlock::Live => CauseKind::LivePath,
        ArgBlock::Omitted => CauseKind::OmittedArgument,
        ArgBlock::Variable => CauseKind::UnprovenVariable,
        ArgBlock::Forwarded => CauseKind::ForwardedParameter,
    };
    Cause::new(kind, caller.file.clone(), caller.line, text)
}

fn not_named(site: &ResolvedCallSite, origin: &PathOrigin) -> Cause {
    let line = origin.owner_line.unwrap_or(1);
    Cause::new(CauseKind::NotNamedFunction, site.file.clone(), line, "")
}

#[cfg(test)]
#[path = "parameter_cause/tests.rs"]
mod tests;
