use super::*;
use crate::codebase::dependencies::extract::{CallableId, ValueCall};

type CallsByOwner<'a> = FxHashMap<(u32, Option<CallableId>), Vec<&'a ValueCall>>;
type CallsByOffset<'a> = FxHashMap<u32, Vec<&'a ValueCall>>;

pub(in super::super) struct CallIndex<'a> {
    files: FxHashMap<std::path::PathBuf, FileCalls<'a>>,
}

struct FileCalls<'a> {
    owners: CallsByOwner<'a>,
    lexical: CallsByOffset<'a>,
}

impl<'a> CallIndex<'a> {
    pub(in super::super) fn new(
        sites: &FxHashMap<NodeId, Vec<&ResolvedCallSite>>,
        facts: &'a dyn TsFactLookup,
    ) -> Self {
        let files: FxHashSet<_> = sites.keys().filter_map(NodeId::as_file).collect();
        let indexed = files
            .into_iter()
            .filter_map(|path| {
                let file = facts.get_ts_facts(path)?;
                let mut calls: CallsByOwner<'a> = FxHashMap::default();
                let mut lexical: CallsByOffset<'a> = FxHashMap::default();
                for call in &file.value_flow.calls {
                    calls
                        .entry((call.offset, call.context))
                        .or_default()
                        .push(call);
                    if TIMERS.contains(&call.source_callee.as_str()) {
                        lexical.entry(call.offset).or_default().push(call);
                    }
                }
                Some((
                    path.to_path_buf(),
                    FileCalls {
                        owners: calls,
                        lexical,
                    },
                ))
            })
            .collect();
        Self { files: indexed }
    }

    pub(in super::super) fn at(&self, site: &ResolvedCallSite) -> &[&'a ValueCall] {
        self.files
            .get(&site.file)
            .and_then(|calls| calls.owners.get(&(site.offset, site.caller_id)))
            .map_or(&[], Vec::as_slice)
    }

    pub(in super::super) fn at_lexical_call(&self, site: &ResolvedCallSite) -> &[&'a ValueCall] {
        self.files
            .get(&site.file)
            .and_then(|calls| calls.lexical.get(&site.offset))
            .map_or(&[], Vec::as_slice)
    }
}
