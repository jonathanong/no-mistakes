use super::Escapes;
use crate::fx::FxHashSet;
use std::path::Path;

impl Escapes {
    pub(in crate::codebase::rules::test_no_live_repository_scan) fn with_local(
        file: &Path,
        name: &str,
    ) -> Self {
        let mut escapes = Self::default();
        let mut names = FxHashSet::default();
        names.insert(name.to_string());
        escapes.local.insert(file.to_path_buf(), names);
        escapes
    }

    pub(in crate::codebase::rules::test_no_live_repository_scan) fn with_reference(
        file: &Path,
        name: &str,
        line: u32,
    ) -> Self {
        let mut escapes = Self::with_local(file, name);
        escapes
            .refs
            .push((file.to_path_buf(), 1, line, name.to_string()));
        escapes
    }
}

#[cfg(test)]
mod dropped_load {
    use super::super::Escapes;
    use crate::codebase::dependencies::graph::TsFactLookup;
    use crate::codebase::ts_source::facts::TsFileFacts;
    use crate::codebase::ts_source::module_facts::arguments::PathProvenance;
    use crate::codebase::ts_source::module_facts::{
        ModuleDiagnostic, ModuleSpan, TypeScriptModuleFacts,
    };
    use crate::fx::FxHashSet;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    struct OneFile {
        file: PathBuf,
        facts: TsFileFacts,
    }

    impl TsFactLookup for OneFile {
        fn get_ts_facts(&self, path: &Path) -> Option<&TsFileFacts> {
            (path == self.file).then_some(&self.facts)
        }
    }

    fn facts(unknown: bool, line: Option<u32>) -> TsFileFacts {
        let provenance = PathProvenance {
            modules_unknown: unknown,
            unknown_load: line,
            ..PathProvenance::default()
        };
        TsFileFacts {
            module_bindings: Some(Arc::new(TypeScriptModuleFacts {
                path_provenance: Some(Arc::new(provenance)),
                diagnostics: vec![ModuleDiagnostic {
                    kind: "unsupported".into(),
                    message: "CommonJS module/export objects are unsupported".into(),
                    span: None,
                }],
                ..TypeScriptModuleFacts::default()
            })),
            ..TsFileFacts::default()
        }
    }

    fn sites(file: &str, facts: TsFileFacts) -> Vec<(PathBuf, u32)> {
        let file = PathBuf::from(file);
        let lookup = OneFile {
            facts,
            file: file.clone(),
        };
        let reachable = FxHashSet::from_iter([file.clone()]);
        Escapes::new(&lookup, &[file], &reachable)
            .unknown_sites()
            .to_vec()
    }

    #[test]
    fn a_dropped_provenance_keeps_the_unreadable_load_line() {
        let file = PathBuf::from("/repo/src/unreadable.cjs");
        assert_eq!(
            sites(file.to_str().unwrap(), facts(true, Some(4))),
            vec![(file.clone(), 4)]
        );
        assert_eq!(
            sites(&file.to_string_lossy(), facts(true, None)),
            vec![(file, 1)]
        );
    }

    #[test]
    fn dropped_facts_without_an_unreadable_load_stay_quiet() {
        let file = "/repo/src/fn.mts";
        assert!(sites(file, facts(false, None)).is_empty());
        assert!(sites(file, TsFileFacts::default()).is_empty());
        let mut quiet = facts(false, None);
        if let Some(module) = quiet.module_bindings.as_mut() {
            let module = Arc::make_mut(module);
            module.path_provenance = None;
        }
        assert!(sites(file, quiet).is_empty());
    }

    fn load(message: &str, span: Option<ModuleSpan>) -> ModuleDiagnostic {
        ModuleDiagnostic {
            kind: "unsupported".into(),
            message: message.into(),
            span,
        }
    }

    #[test]
    fn a_commonjs_file_keeps_the_computed_require_line() {
        let source = "\nmodule.exports = require(name);\n";
        let require_at = u32::try_from(source.find("require").unwrap()).unwrap();
        let file = "/repo/src/unreadable.cjs";
        let require_span = ModuleSpan {
            start: require_at,
            end: require_at + 7,
        };
        let mut bindings = TypeScriptModuleFacts {
            diagnostics: vec![
                ModuleDiagnostic {
                    kind: "semanticError".into(),
                    message: "ignored".into(),
                    span: None,
                },
                load(
                    "CommonJS module/export objects are unsupported",
                    Some(ModuleSpan { start: 1, end: 7 }),
                ),
                load(
                    "Non-literal or unsupported require specifier",
                    Some(require_span),
                ),
            ],
            ..TypeScriptModuleFacts::default()
        };
        let at_require = TsFileFacts {
            source: Some(source.into()),
            module_bindings: Some(Arc::new(bindings.clone())),
            ..TsFileFacts::default()
        };
        assert_eq!(sites(file, at_require), vec![(PathBuf::from(file), 2)]);
        bindings
            .diagnostics
            .push(load("Non-literal dynamic import specifier", None));
        bindings.diagnostics.push(load(
            "Shadowed require is not a module loader",
            Some(require_span),
        ));
        let earliest = TsFileFacts {
            source: Some(source.into()),
            module_bindings: Some(Arc::new(bindings)),
            ..TsFileFacts::default()
        };
        assert_eq!(sites(file, earliest), vec![(PathBuf::from(file), 1)]);
        let mut missing_source = facts(false, None);
        let module = Arc::make_mut(missing_source.module_bindings.as_mut().unwrap());
        module.path_provenance = None;
        module.diagnostics.push(load(
            "Indirect require reference is unsupported",
            Some(ModuleSpan { start: 4, end: 8 }),
        ));
        assert_eq!(sites(file, missing_source), vec![(PathBuf::from(file), 1)]);
    }
}
