//! One caller of a parameter root: seed, block, or follow that caller's parameter.
use super::super::cause::{Cause, CauseKind};
use super::super::parameters::{argument_path, argument_present, argument_text, unproven_argument};
use super::super::{path_provenance, Owners};
use super::Step;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::test_no_live_repository_scan::provenance::{
    arg_block, classify, live_or_variable, missing_block, ArgBlock, Verdict,
};
use crate::codebase::ts_source::module_facts::arguments::{ArgumentText, PathProvenance};

impl Owners<'_> {
    pub(super) fn caller_step(
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
        let text = argument_text(provenance, caller.offset, index, member);
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
            if !argument
                .root_tails
                .iter()
                .all(|tail| self.safe_suffix(tail, caller, facts, 0))
                || !self.env_argument_owned(name, caller, facts)
            {
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

fn caller_cause(caller: &ResolvedCallSite, block: ArgBlock, text: Option<&ArgumentText>) -> Cause {
    let kind = match block {
        ArgBlock::Live => CauseKind::LivePath,
        ArgBlock::Omitted => CauseKind::OmittedArgument,
        ArgBlock::Variable => CauseKind::UnprovenVariable,
        ArgBlock::Forwarded => CauseKind::ForwardedParameter,
    };
    Cause::new(
        kind,
        caller.file.clone(),
        text.map_or(caller.line, |argument| argument.line),
        text.map_or("", |argument| argument.text.as_str()),
    )
    .with_property(text.and_then(|argument| argument.property.clone()))
}
