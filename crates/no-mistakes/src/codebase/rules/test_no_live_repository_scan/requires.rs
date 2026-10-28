//! CommonJS `require` of an enumerating module whose uses the call graph cannot resolve.
//!
//! `const { readdirSync: ls } = require("fs")` or `require("glob").sync` bind the
//! enumerator under a name the graph does not follow. Such a require is reported as
//! an unresolved root rather than skipped, unless a call in the same file already
//! resolved to that module.
use super::config::Enumerator;
use super::enumerators::{enumerates, FS_MODULES, GLOB_MODULES};
use super::findings;
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite, TsFactLookup};
use crate::codebase::rules::RuleFinding;
use crate::config::v2::schema::RuleDef;
use crate::fx::FxHashSet;
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
    /// Modules some call in the file already resolved to as an enumerator.
    resolved: FxHashSet<String>,
}

#[derive(Default)]
pub(super) struct Sites(BTreeMap<PathBuf, FileSites>);

impl Sites {
    pub(super) fn visit(&mut self, site: &ResolvedCallSite, extra: &[Enumerator]) {
        let file = self.0.entry(site.file.clone()).or_default();
        for target in site.targets() {
            if let KnownCallTargetRef::ModuleExport {
                specifier,
                export_path,
                ..
            } = target.target
            {
                if enumerates(specifier, export_path, extra) {
                    file.resolved.insert(plain(specifier).to_owned());
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
            for import in ts.imports.iter().filter(|i| i.kind == ImportKind::Require) {
                let module = plain(&import.specifier);
                if import.computed
                    || !is_enumerator_module(module, extra)
                    || sites.resolved.contains(module)
                {
                    continue;
                }
                found.push(findings::unresolved_require(
                    root,
                    application,
                    file,
                    import.line,
                    &import.specifier,
                ));
            }
        }
        found
    }
}
