//! CommonJS `require` of an enumerating module whose uses the call graph cannot resolve.
//!
//! `const { readdirSync: ls } = require("fs")` or `require("glob").sync` bind the
//! enumerator under a name the graph does not follow. A require is reported as
//! an unresolved root unless every binding from that load has a resolved scan.
use super::config::Enumerator;
use super::enumerators::{enumerates, scan_of, Scan, FS_MODULES, GLOB_MODULES};
use super::findings;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::module_facts::RequireBinding;
use crate::config::v2::schema::RuleDef;
use crate::fx::{FxHashMap, FxHashSet};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn plain(module: &str) -> &str {
    module.strip_prefix("node:").unwrap_or(module)
}

fn is_enumerator_module(specifier: &str, extra: &[Enumerator]) -> bool {
    let module = plain(specifier);
    FS_MODULES.contains(&module)
        || GLOB_MODULES.contains(&module)
        || extra.iter().any(|extra| plain(&extra.module) == module)
}

/// What the reachable call sites of one file revealed.
#[derive(Default)]
pub(super) struct FileSites {
    /// The local binding and module of each resolved enumerating call.
    resolved: FxHashMap<String, FxHashSet<String>>,
    /// Unknown direct member scans already reported at the load's own offset.
    unresolved_direct_offsets: FxHashSet<u32>,
}

#[derive(Default)]
pub(super) struct Sites(BTreeMap<PathBuf, FileSites>);

impl Sites {
    pub(super) fn visit(&mut self, site: &ResolvedCallSite, extra: &[Enumerator]) {
        let file = self.0.entry(site.file.clone()).or_default();
        if matches!(scan_of(site, extra), Some(Scan::Unresolved)) {
            file.unresolved_direct_offsets.insert(site.offset);
        }
        for target in site.targets() {
            if let KnownCallTargetRef::ModuleExport {
                specifier,
                export_path,
                ..
            } = target.target
            {
                if enumerates(specifier, export_path, extra) {
                    let local = site.source_callee.split('.').next().unwrap_or_default();
                    file.resolved
                        .entry(plain(specifier).to_owned())
                        .or_default()
                        .insert(local.to_owned());
                }
            }
        }
    }

    pub(super) fn unresolved_requires(
        &self,
        root: &Path,
        application: &RuleDef,
        facts: &dyn TsFactLookup,
        extra: &[Enumerator],
    ) -> Vec<RuleFinding> {
        let mut found = Vec::new();
        for (file, sites) in &self.0 {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            let mut loads_by_module: FxHashMap<&str, Vec<u32>> = FxHashMap::default();
            let mut bindings_by_load: FxHashMap<u32, Vec<&RequireBinding>> = FxHashMap::default();
            let mut root_scope = None;
            if let Some(module_facts) = ts.module_bindings.as_ref() {
                root_scope = module_facts
                    .scopes
                    .iter()
                    .find(|scope| scope.parent_id.is_none())
                    .map(|scope| scope.id);
                for load in module_facts
                    .loads
                    .iter()
                    .filter(|load| load.kind == "require")
                {
                    loads_by_module
                        .entry(plain(&load.specifier))
                        .or_default()
                        .push(load.span.start);
                }
                for binding in &module_facts.require_bindings {
                    bindings_by_load
                        .entry(binding.load_start)
                        .or_default()
                        .push(binding);
                }
            }
            for loads in loads_by_module.values_mut() {
                loads.sort_unstable();
            }
            let mut next_load: FxHashMap<&str, usize> = FxHashMap::default();
            let requires = ts.imports.iter().filter(|i| i.kind == ImportKind::Require);
            for import in requires {
                let module = plain(&import.specifier);
                if !import.computed && !is_enumerator_module(module, extra) {
                    continue;
                }
                if !import.computed {
                    let index = next_load.entry(module).or_default();
                    let load = loads_by_module
                        .get(module)
                        .and_then(|loads| loads.get(*index));
                    *index += 1;
                    if load.is_some_and(|load| sites.unresolved_direct_offsets.contains(load)) {
                        continue;
                    }
                    if let Some(bindings) = load.and_then(|load| bindings_by_load.get(load)) {
                        let relevant: Vec<_> = bindings
                            .iter()
                            .filter(|binding| {
                                binding
                                    .imported
                                    .as_deref()
                                    .is_none_or(|path| enumerates(module, path, extra))
                            })
                            .collect();
                        if relevant.is_empty()
                            || relevant.iter().all(|binding| {
                                Some(binding.scope_id) == root_scope
                                    && sites.resolved.get(module).is_some_and(|locals| {
                                        locals.contains(binding.local.as_str())
                                    })
                            })
                        {
                            continue;
                        }
                    }
                }
                found.push(findings::unresolved_require(
                    root,
                    application,
                    file,
                    import.line,
                    &import.specifier,
                    import.computed,
                ));
            }
        }
        found
    }
}
