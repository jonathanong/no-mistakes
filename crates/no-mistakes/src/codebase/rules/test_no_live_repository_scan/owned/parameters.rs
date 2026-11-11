use super::{env, path_provenance, Owners};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{
    ArgumentText, PathDefault, PathOrigin, PathProvenance,
};

pub(super) struct Parameter<'a> {
    pub(super) index: usize,
    pub(super) member: Option<&'a str>,
    pub(super) default: Option<&'a PathDefault>,
    pub(super) relative: bool,
}

impl Owners<'_> {
    pub(super) fn env_argument_owned(
        &self,
        name: &str,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> bool {
        if !self.env.all_owned(name) || self.overlapping_env(&site.file, site.offset, name, facts) {
            return false;
        }
        facts
            .get_ts_facts(&site.file)
            .and_then(path_provenance)
            .is_some_and(|provenance| {
                if env::cleanup_read(provenance, name, site.offset) {
                    return false;
                }
                env::dominated(provenance, name, site.offset, &self.fixture_dirs)
                    || self.imported.owns(
                        &site.file,
                        name,
                        facts,
                        &self.consumed,
                        &self.fixture_dirs,
                    )
                    || self.setups.owns(
                        &site.file,
                        name,
                        facts,
                        &self.imported,
                        &self.consumed,
                        &self.fixture_dirs,
                    )
            })
    }

    pub(super) fn parameter_source(
        &self,
        origin: &PathOrigin,
        site: &ResolvedCallSite,
    ) -> Option<NodeId> {
        let offset = origin.param_owner?;
        self.nodes.get(&site.file)?.get(&offset).cloned()
    }
}

pub(super) fn argument_path<'a>(
    paths: &'a PathProvenance,
    offset: u32,
    index: usize,
    member: Option<&str>,
) -> Option<&'a PathOrigin> {
    match member {
        Some(member) => paths
            .arg_member_paths
            .get(&offset)?
            .iter()
            .find(|(position, key, _)| *position == index && key == member)
            .map(|(_, _, origin)| origin),
        None => paths
            .arg_paths
            .get(&offset)?
            .iter()
            .find(|(position, _)| *position == index)
            .map(|(_, origin)| origin),
    }
}

pub(super) fn argument_text<'a>(
    paths: &'a PathProvenance,
    offset: u32,
    index: usize,
    member: Option<&str>,
) -> Option<&'a ArgumentText> {
    let args = paths.arg_text.get(&offset)?;
    args.iter()
        .find(|arg| arg.index == index && arg.property.as_deref() == member)
        .or_else(|| {
            args.iter()
                .find(|arg| arg.index == index && arg.property.is_none())
        })
}

pub(super) fn argument_present(paths: &PathProvenance, offset: u32, index: usize) -> bool {
    paths.arg_text.get(&offset).is_some_and(|args| {
        args.iter().any(|arg| {
            arg.property.is_none()
                && (arg.index == index || (arg.index <= index && arg.text.starts_with("...")))
        })
    })
}

pub(super) fn unproven_argument(
    paths: &PathProvenance,
    offset: u32,
    index: usize,
) -> Option<&PathOrigin> {
    paths
        .arg_unproven
        .get(&offset)
        .and_then(|args| args.iter().find(|(position, _)| *position == index))
        .map(|(_, origin)| origin)
}

pub(super) fn argument_omitted(paths: &PathProvenance, offset: u32, index: usize) -> bool {
    paths
        .arg_counts
        .get(&offset)
        .copied()
        .flatten()
        .is_some_and(|count| index >= count)
}

pub(super) fn argument_missing_member(
    paths: &PathProvenance,
    offset: u32,
    index: usize,
    member: &str,
    nullish: bool,
) -> bool {
    let mut closed = paths
        .arg_closed_objects
        .get(&offset)
        .is_some_and(|positions| positions.contains(&index));
    let mut prefix = String::new();
    for part in member.split('.') {
        if !prefix.is_empty() {
            prefix.push('.');
        }
        prefix.push_str(part);
        let Some(origin) = argument_path(paths, offset, index, Some(&prefix)) else {
            return closed;
        };
        if nullish && origin.nullish {
            return true;
        }
        closed = paths
            .arg_closed_members
            .get(&offset)
            .is_some_and(|members| members.contains(&(index, prefix.clone())));
    }
    false
}
