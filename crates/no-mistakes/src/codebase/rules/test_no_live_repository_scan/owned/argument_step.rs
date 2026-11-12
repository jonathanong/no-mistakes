use super::super::provenance::{arg_block, classify, live_or_variable, ArgBlock, Verdict};
use super::parameter_cause::step::caller_cause;
use super::parameter_cause::Step;
use super::Owners;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentText, PathOrigin, PathValueKind,
};

impl Owners<'_> {
    pub(super) fn argument_step(
        &self,
        argument: &PathOrigin,
        caller: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
        text: Option<&ArgumentText>,
    ) -> Step {
        if argument.require_scalar
            && argument.value_kind != PathValueKind::Scalar
            && (argument.value_kind == PathValueKind::Mutable || argument.param.is_none())
        {
            return Step::Block(caller_cause(caller, ArgBlock::Variable, text));
        }
        if argument.relative && argument.environment().is_some() {
            return Step::Block(caller_cause(caller, ArgBlock::Variable, text));
        }
        if let Some(block) = arg_block(argument, fixture_dirs) {
            return Step::Block(caller_cause(caller, block, text));
        }
        if classify(Some(argument), fixture_dirs) == Verdict::Allowed {
            return Step::Seed;
        }
        if let Some((name, capture)) = argument.environment() {
            if !argument
                .root_tails
                .iter()
                .all(|tail| self.safe_suffix(tail, caller, facts, 0))
                || !self.env_argument_owned(name, capture, caller, facts)
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
            argument.param_default.clone(),
            argument.relative,
            argument.require_scalar && argument.value_kind != PathValueKind::Scalar,
        )
    }
}
