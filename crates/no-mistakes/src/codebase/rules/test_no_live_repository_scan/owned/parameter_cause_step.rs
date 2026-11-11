//! One caller of a parameter root: seed, block, or follow that caller's parameter.
use super::super::cause::{Cause, CauseKind};
use super::super::parameters::{
    argument_omitted, argument_path, argument_present, argument_text, unproven_argument,
};
use super::super::{path_provenance, Owners};
use super::Step;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::test_no_live_repository_scan::provenance::{
    arg_block, classify, live_or_variable, missing_block, ArgBlock, Verdict,
};
use crate::codebase::ts_source::module_facts::arguments::{ArgumentText, PathOrigin, PathProvenance};

impl Owners<'_> {
    pub(super) fn caller_steps(
        &self,
        caller: &ResolvedCallSite,
        index: usize,
        member: Option<&str>,
        default: Option<&PathOrigin>,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Vec<Step> {
        let empty = PathProvenance::default();
        let provenance = facts
            .get_ts_facts(&caller.file)
            .and_then(path_provenance)
            .unwrap_or(&empty);
        let argument = argument_path(provenance, caller.offset, index, member);
        let forwarded = member.and_then(|member| {
            let origin = argument_path(provenance, caller.offset, index, None)?;
            let atoms = origin
                .atoms()
                .iter()
                .map(|atom| {
                    if atom.param.is_none() || atom.param_member.is_some() {
                        return None;
                    }
                    let mut atom = atom.clone();
                    atom.param_member = Some(member.to_owned());
                    atom.param_default = default.cloned().map(Box::new);
                    Some(atom)
                })
                .collect::<Option<Vec<_>>>()?;
            Some(PathOrigin::alternatives(atoms))
        });
        let argument = argument.or(forwarded.as_ref());
        let present = argument_present(provenance, caller.offset, index);
        let text = argument_text(provenance, caller.offset, index, member);
        let closed = provenance
            .arg_closed_objects
            .get(&caller.offset)
            .is_some_and(|positions| positions.contains(&index));
        let argument = argument.or_else(|| {
            (argument_omitted(provenance, caller.offset, index) || (member.is_some() && closed))
                .then_some(default)
                .flatten()
        });
        if argument.is_none() {
            if let Some(observed) = unproven_argument(provenance, caller.offset, index) {
                if matches!(classify(Some(observed), fixture_dirs), Verdict::Live(_)) {
                    return vec![Step::Block(caller_cause(caller, ArgBlock::Live, text))];
                }
            }
        }
        let Some(argument) = argument else {
            return vec![Step::Block(caller_cause(
                caller,
                missing_block(present),
                text,
            ))];
        };
        argument
            .atoms()
            .iter()
            .map(|argument| {
                let step = self.argument_step(argument, caller, facts, fixture_dirs, text.clone());
                if !present && matches!(step, Step::Block(_)) {
                    Step::Block(caller_cause(caller, ArgBlock::Omitted, text.clone()))
                } else {
                    step
                }
            })
            .collect()
    }

    fn argument_step(
        &self,
        argument: &PathOrigin,
        caller: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
        text: Option<&ArgumentText>,
    ) -> Step {
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
            argument.param_default.clone(),
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
