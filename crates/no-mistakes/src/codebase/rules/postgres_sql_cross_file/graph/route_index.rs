use super::resolve::*;
use crate::codebase::dependencies::extract::{
    CallTargetIdentity, FunctionCall, ImportedBinding, ValueBinding,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, fx_set, FxHashMap, FxHashSet};
use std::cell::OnceCell;

/// Borrowed fact indexes and canonical-edge projections, owned by one rule scan.
pub(crate) struct RegistryRoutes<'a> {
    pub(super) facts: &'a CheckFactMap,
    files: OnceCell<FxHashMap<&'a Path, OnceCell<FileRoutes<'a>>>>,
    #[cfg(feature = "test-instrumentation")]
    pub(crate) work: std::cell::Cell<[usize; 3]>,
}
pub(super) struct FileRoutes<'a> {
    ts: &'a TsFileFacts,
    imports: OnceCell<FxHashMap<&'a str, &'a ImportedBinding>>,
    exports: OnceCell<Exports<'a>>,
    requires: OnceCell<Requires<'a>>,
    pub targets: [OnceCell<Option<(String, PathBuf)>>; 2],
}
#[derive(Default)]
pub(super) struct Exports<'a> {
    pub named: FxHashMap<&'a str, &'a ExportedBinding>,
    pub namespaces: FxHashMap<&'a str, &'a ExportedBinding>,
}
#[derive(Default)]
pub(super) struct Requires<'a> {
    pub bindings: FxHashMap<&'a str, &'a ValueBinding>,
    pub calls: FxHashMap<u32, &'a FunctionCall>,
    pub eager: FxHashSet<&'a str>,
}
impl<'a> RegistryRoutes<'a> {
    pub(crate) fn new(facts: &'a CheckFactMap) -> Self {
        Self {
            facts,
            files: OnceCell::new(),
            #[cfg(feature = "test-instrumentation")]
            work: std::cell::Cell::new([0; 3]),
        }
    }
    pub(super) fn file(&self, file: &Path) -> Option<&FileRoutes<'a>> {
        let ts = self.facts.get_ts_facts(file)?;
        let files = self.files.get_or_init(|| {
            self.facts
                .ts
                .keys()
                .map(|path| (path.as_path(), OnceCell::new()))
                .collect()
        });
        Some(files.get(file)?.get_or_init(|| {
            self.record(0);
            FileRoutes {
                ts,
                imports: OnceCell::new(),
                exports: OnceCell::new(),
                requires: OnceCell::new(),
                targets: [OnceCell::new(), OnceCell::new()],
            }
        }))
    }
    pub(super) fn record(&self, slot: usize) {
        #[cfg(feature = "test-instrumentation")]
        {
            let mut work = self.work.get();
            work[slot] += 1;
            self.work.set(work);
        }
        #[cfg(not(feature = "test-instrumentation"))]
        let _ = slot;
    }
}
impl<'a> FileRoutes<'a> {
    pub(super) fn imports(
        &self,
        routes: &RegistryRoutes<'_>,
    ) -> &FxHashMap<&'a str, &'a ImportedBinding> {
        self.imports.get_or_init(|| {
            let mut result = fx_map();
            if self.ts.parse_error.is_none() && self.ts.operational_error.is_none() {
                for binding in &self.ts.imported_bindings {
                    routes.record(1);
                    if !binding.is_type_only {
                        result.entry(binding.local.as_str()).or_insert(binding);
                    }
                }
            }
            result
        })
    }
    pub(super) fn exports(&self, routes: &RegistryRoutes<'_>) -> &Exports<'a> {
        self.exports.get_or_init(|| {
            let mut result = Exports::default();
            for binding in &self.ts.exported_bindings {
                routes.record(1);
                result.named.entry(&binding.exported).or_insert(binding);
                if binding.local == "*" {
                    result
                        .namespaces
                        .entry(&binding.exported)
                        .or_insert(binding);
                }
            }
            result
        })
    }
    pub(super) fn requires(&self, routes: &RegistryRoutes<'_>) -> &Requires<'a> {
        self.requires.get_or_init(|| {
            let mut result = Requires {
                eager: fx_set(),
                ..Default::default()
            };
            // Every supported require route needs this proof; ordinary ESM files
            // do not demand value-binding or raw-call indexes at all.
            if self.ts.eager_require_specifiers.is_empty() {
                return result;
            }
            for binding in &self.ts.value_flow.bindings {
                routes.record(1);
                if binding.owner.is_none() && binding.scope == 0 && binding.immutable {
                    result.bindings.entry(&binding.name).or_insert(binding);
                }
            }
            for call in &self.ts.function_calls {
                routes.record(1);
                if call.callee == "require"
                    && call.target_identity == CallTargetIdentity::Global
                    && call.callee_binding_scope.is_none()
                {
                    result.calls.entry(call.offset).or_insert(call);
                }
            }
            for specifier in &self.ts.eager_require_specifiers {
                routes.record(1);
                result.eager.insert(specifier);
            }
            result
        })
    }
}
