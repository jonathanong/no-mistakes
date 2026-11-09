use super::*;
use crate::codebase::dependencies::extract::{CallableId, ValueCall};

type CallsByOwner<'a> = FxHashMap<(u32, Option<CallableId>), Vec<&'a ValueCall>>;

pub(in super::super) struct CallIndex<'a> {
    files: FxHashMap<std::path::PathBuf, CallsByOwner<'a>>,
}

impl<'a> CallIndex<'a> {
    pub(in super::super) fn new(
        sites: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>,
        facts: &'a dyn TsFactLookup,
    ) -> Self {
        let files: FxHashSet<_> = sites.keys().filter_map(NodeId::as_file).collect();
        let mut indexed = FxHashMap::default();
        for path in files {
            let Some(file) = facts.get_ts_facts(path) else {
                continue;
            };
            let mut calls: CallsByOwner<'a> = FxHashMap::default();
            for call in &file.value_flow.calls {
                calls
                    .entry((call.offset, call.context))
                    .or_default()
                    .push(call);
            }
            indexed.insert(path.to_path_buf(), calls);
        }
        Self { files: indexed }
    }

    pub(in super::super) fn at(&self, site: &ResolvedCallSite) -> &[&'a ValueCall] {
        self.files
            .get(&site.file)
            .and_then(|calls| calls.get(&(site.offset, site.caller_id)))
            .map_or(&[], Vec::as_slice)
    }
}
