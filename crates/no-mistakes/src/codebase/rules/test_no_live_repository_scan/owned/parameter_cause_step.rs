//! One caller of a parameter root: seed, block, or follow that caller's parameter.
use super::super::cause::{Cause, CauseKind};
use super::super::parameters::{
    argument_missing_member, argument_omitted, argument_path, argument_present, argument_text,
    unproven_argument, Parameter,
};
use super::super::{path_provenance, Owners};
use super::Step;
use crate::codebase::dependencies::graph::{ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::test_no_live_repository_scan::provenance::{
    arg_block, classify, live_or_variable, missing_block, ArgBlock, Verdict,
};
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentText, PathOrigin, PathProvenance,
};

impl Owners<'_> {
    pub(in super::super) fn caller_steps(
        &self,
        caller: &ResolvedCallSite,
        parameter: Parameter<'_>,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Vec<Step> {
        let Parameter {
            index,
            member,
            default,
            relative,
        } = parameter;
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
                    if atom.param.is_none()
                        || atom.param_member.is_some()
                        || atom.param_default.is_some()
                    {
                        return None;
                    }
                    let mut atom = atom.clone();
                    atom.param_member = Some(member.to_owned());
                    atom.param_default = default.map(|default| {
                        super::super::defaults::forward_default(default, index, &atom)
                    });
                    Some(atom)
                })
                .collect::<Option<Vec<_>>>()?;
            Some(PathOrigin::alternatives(atoms))
        });
        let argument = argument.or(forwarded.as_ref());
        let present = argument_present(provenance, caller.offset, index);
        let text = argument_text(provenance, caller.offset, index, member);
        let use_default = argument.is_none()
            && (argument_omitted(provenance, caller.offset, index)
                || member.is_some_and(|member| {
                    argument_missing_member(
                        provenance,
                        caller.offset,
                        index,
                        member,
                        default.is_some_and(|fallback| fallback.nullish),
                    )
                }))
            || argument.is_some_and(|argument| argument.nullish)
                && default.is_some_and(|fallback| fallback.nullish);
        if use_default {
            if let Some(default) = default {
                let steps = self.fallback_steps(
                    default,
                    caller,
                    Parameter {
                        index,
                        member,
                        default: Some(default),
                        relative,
                    },
                    facts,
                    fixture_dirs,
                );
                return steps
                    .into_iter()
                    .map(|step| {
                        if !present && matches!(step, Step::Block(_)) {
                            Step::Block(caller_cause(caller, ArgBlock::Omitted, text))
                        } else {
                            step
                        }
                    })
                    .collect();
            }
        }
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
                let mut argument = argument.clone();
                argument.relative |= relative;
                let step = self.argument_step(&argument, caller, facts, fixture_dirs, text);
                if !present && matches!(step, Step::Block(_)) {
                    Step::Block(caller_cause(caller, ArgBlock::Omitted, text))
                } else {
                    step
                }
            })
            .collect()
    }

    pub(in super::super) fn argument_step(
        &self,
        argument: &PathOrigin,
        caller: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
        text: Option<&ArgumentText>,
    ) -> Step {
        if argument.relative && argument.env.is_some() {
            return Step::Block(caller_cause(caller, ArgBlock::Variable, text));
        }
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
            argument.relative,
        )
    }
}

pub(in super::super) fn caller_cause(
    caller: &ResolvedCallSite,
    block: ArgBlock,
    text: Option<&ArgumentText>,
) -> Cause {
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

pub(super) fn not_named(site: &ResolvedCallSite, origin: &PathOrigin) -> Cause {
    let line = origin.owner_line.unwrap_or(1);
    Cause::new(CauseKind::NotNamedFunction, site.file.clone(), line, "")
}
