use super::{env, path_provenance, Owners};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::{PathOrigin, PathProvenance};

impl Owners<'_> {
    pub(super) fn env_argument_owned(
        &self,
        name: &str,
        site: &ResolvedCallSite,
        facts: &dyn TsFactLookup,
    ) -> bool {
        if !self.env.all_owned(name) {
            return false;
        }
        facts
            .get_ts_facts(&site.file)
            .and_then(path_provenance)
            .is_some_and(|provenance| {
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

pub(super) fn argument_text(paths: &PathProvenance, offset: u32, index: usize) -> String {
    paths
        .arg_text
        .get(&offset)
        .and_then(|args| args.iter().find(|(position, _)| *position == index))
        .map(|(_, text)| text.clone())
        .unwrap_or_default()
}

pub(super) fn argument_present(paths: &PathProvenance, offset: u32, index: usize) -> bool {
    paths
        .arg_text
        .get(&offset)
        .is_some_and(|args| args.iter().any(|(position, _)| *position == index))
}

pub(super) fn unproven_argument<'a>(
    paths: &'a PathProvenance,
    offset: u32,
    index: usize,
) -> Option<&'a PathOrigin> {
    paths
        .arg_unproven
        .get(&offset)
        .and_then(|args| args.iter().find(|(position, _)| *position == index))
        .map(|(_, origin)| origin)
}
