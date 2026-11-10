//! The parameter-root walk. A failure names the caller or route that stopped it.
use super::cause::{function_site, line_at, Cause, CauseKind, Ownership};
use super::parameters::{argument_path, argument_present, argument_text, unproven_argument};
use super::{path_provenance, Owners};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::test_no_live_repository_scan::provenance::{
    arg_block, classify, ArgBlock, Verdict,
};
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
            (Some(name), _) => self
                .env_owned(name, site, facts)
                .then_some(Ownership::Yes)
                .unwrap_or(Ownership::No),
            (None, Some(index)) => match self.param_cause(index, origin, site, facts, fixture_dirs)
            {
                Ok(()) => Ownership::Yes,
                Err(cause) => Ownership::Parameter(cause),
            },
            (None, None) => origin.reassigned_at.map_or(Ownership::No, |at| {
                Ownership::Parameter(Cause::new(
                    CauseKind::ReassignedParameter,
                    site.file.clone(),
                    line_at(facts, &site.file, at),
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
        let Some(source) = self.parameter_source(origin, site) else {
            return Err(Cause::new(
                CauseKind::NotNamedFunction,
                site.file.clone(),
                line_at(facts, &site.file, origin.param_owner.unwrap_or(site.offset)),
                "",
            ));
        };
        let start = source.clone();
        let mut pending = vec![(source, index, origin.param_member.clone())];
        let mut seen = FxHashSet::default();
        let mut seeded = false;
        let mut forwarded = None;
        while let Some((source, index, member)) = pending.pop() {
            if !seen.insert((source.clone(), index, member.clone())) {
                continue;
            }
            if self.escape_unknown || !self.proven_target(&source) {
                return Err(self.route_cause(&source, facts));
            }
            let Some(callers) = self
                .callers
                .get(&source)
                .filter(|callers| !callers.is_empty())
            else {
                let (file, line) = function_site(facts, &source);
                return Err(Cause::new(CauseKind::NoCaller, file, line, ""));
            };
            for caller in callers.iter().copied() {
                let step = self.caller_step(caller, index, member.as_deref(), facts, fixture_dirs);
                match step {
                    Step::Block(cause) => return Err(cause),
                    Step::Seed => seeded = true,
                    Step::Follow(next, next_index, next_member, cause) => {
                        if forwarded.is_none() {
                            forwarded = Some(cause);
                        }
                        pending.push((next, next_index, next_member));
                    }
                }
            }
        }
        if seeded {
            Ok(())
        } else {
            Err(forwarded.unwrap_or_else(|| {
                let (file, line) = function_site(facts, &start);
                Cause::new(CauseKind::NoCaller, file, line, "")
            }))
        }
    }
}

enum Step {
    Block(Cause),
    Seed,
    Follow(NodeId, usize, Option<String>, Cause),
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
        let provenance = facts.get_ts_facts(&caller.file).and_then(path_provenance);
        let Some(provenance) = provenance else {
            return Step::Block(Cause::new(
                CauseKind::UnreadModule,
                caller.file.clone(),
                caller.line,
                "",
            ));
        };
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
        if let Some(block) = arg_block(argument, present, fixture_dirs) {
            return Step::Block(caller_cause(caller, block, text));
        }
        let Some(argument) = argument else {
            return Step::Block(caller_cause(caller, ArgBlock::Omitted, text));
        };
        if classify(Some(argument), fixture_dirs) == Verdict::Allowed {
            return Step::Seed;
        }
        if let Some(name) = argument.env.as_deref() {
            if !self.env_argument_owned(name, caller, facts) {
                let block = if matches!(classify(Some(argument), fixture_dirs), Verdict::Live(_)) {
                    ArgBlock::Live
                } else {
                    ArgBlock::Variable
                };
                return Step::Block(caller_cause(caller, block, text));
            }
            return Step::Seed;
        }
        if let Some(next_index) = argument.param {
            let Some(next) = self.parameter_source(argument, caller) else {
                return Step::Block(caller_cause(caller, ArgBlock::Forwarded, text));
            };
            return Step::Follow(
                next,
                next_index,
                argument.param_member.clone(),
                caller_cause(caller, ArgBlock::Forwarded, text),
            );
        }
        Step::Block(caller_cause(caller, ArgBlock::Variable, text))
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
