use super::super::provenance::ArgBlock;
use super::parameter_cause::step::caller_cause;
use super::parameter_cause::Step;
use super::parameters::{argument_text, Parameter};
use super::path_provenance;
use super::Owners;
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{PathDefault, PathOrigin};

pub(super) fn forward_default(
    default: &PathDefault,
    index: usize,
    forwarded: &PathOrigin,
) -> PathDefault {
    fn remap(origin: &PathOrigin, index: usize, forwarded: &PathOrigin) -> PathOrigin {
        let mut origin = origin.clone();
        if let Some(parameter) = origin.param {
            if parameter != index {
                return PathOrigin::unknown();
            }
            origin.param = forwarded.param;
            origin.param_owner = forwarded.param_owner;
            origin.owner_line = forwarded.owner_line;
        }
        origin.alternatives = origin
            .alternatives
            .iter()
            .map(|atom| remap(atom, index, forwarded))
            .collect();
        origin.param_default = origin
            .param_default
            .as_ref()
            .map(|default| forward_default(default, index, forwarded));
        origin
    }
    PathDefault {
        value: Box::new(remap(&default.value, index, forwarded)),
        nullish: default.nullish,
    }
}

impl Owners<'_> {
    pub(super) fn fallback_steps(
        &self,
        default: &PathDefault,
        caller: &ResolvedCallSite,
        parameter: Parameter<'_>,
        facts: &dyn TsFactLookup,
        fixture_dirs: &[&str],
    ) -> Vec<Step> {
        let text = facts
            .get_ts_facts(&caller.file)
            .and_then(path_provenance)
            .and_then(|paths| {
                argument_text(paths, caller.offset, parameter.index, parameter.member)
            });
        default.value.atoms().iter().flat_map(|atom| {
            let mut atom = atom.clone();
            atom.relative |= parameter.relative;
            if let Some(index) = atom.param {
                let same_owner = matches!(&caller.target_node,
                    Some(NodeId::Symbol { callable_id: Some(id), .. }) if Some(id.0) == atom.param_owner);
                if !same_owner || atom.segments.iter().any(|part| part == "..") {
                    return vec![Step::Block(caller_cause(caller, ArgBlock::Forwarded, text))];
                }
                self.caller_steps(caller, Parameter { index, member: atom.param_member.as_deref(), default: atom.param_default.as_ref(), relative: atom.relative }, facts, fixture_dirs)
            } else {
                vec![self.argument_step(&atom, caller, facts, fixture_dirs, text)]
            }
        }).collect()
    }
}
