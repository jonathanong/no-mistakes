#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum MockedModule {
    File(PathBuf),
    Specifier(String),
}

const EXTERNAL_MOCK_SENTINEL: &str = "/__external_mock__";

struct ExternalMock<'a> {
    specifier: Option<&'a str>,
    factory_span: Option<&'a crate::codebase::ts_source::module_facts::ModuleSpan>,
    factories: &'a [NodeId],
    root: &'a Path,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    files: &'a GraphFiles,
    module: &'a crate::codebase::ts_source::module_facts::TypeScriptModuleFacts,
    ready: bool,
}

struct Chain<'a> {
    modules: &'a FxHashSet<MockedModule>,
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn ImportResolution,
    files: &'a GraphFiles,
}

type ExportSeen = FxHashSet<(PathBuf, String)>;

enum Hop {
    Uncertain,
    Miss,
    Next(String, String),
    Stars(Vec<String>),
}

impl DepGraph {
    fn external_replaced_module(&self, input: ExternalMock<'_>) -> Option<(MockedModule, bool)> {
        let specifier = input.specifier?;
        let module_id = match input.resolver.resolve(specifier, input.root) {
            Some(path) => {
                if input.files.visible_path(&path).is_some() {
                    return None;
                }
                MockedModule::File(crate::codebase::ts_resolver::normalize_path(&path))
            }
            None => MockedModule::Specifier(specifier.to_string()),
        };
        let target = Path::new(EXTERNAL_MOCK_SENTINEL);
        let full = input.ready
            && input.root != target
            && self.mock_factory_is_full(input.factories, input.root, target, input.facts)
            && input.factory_span.is_none_or(|span| {
                self.mock_factory_imports_are_full(
                    input.root,
                    span,
                    target,
                    input.module,
                    input.resolver,
                    input.files,
                )
            });
        Some((module_id, full))
    }

    fn calls_through_mocked_modules(
        &self,
        root: &Path,
        modules: &FxHashSet<MockedModule>,
        facts: &dyn TsFactLookup,
        resolver: &dyn ImportResolution,
        files: &GraphFiles,
    ) -> FxHashSet<(PathBuf, u32)> {
        let chain = Chain {
            modules,
            facts,
            resolver,
            files,
        };
        let mut suppressed = fx_set();
        for site in self.resolved_call_sites() {
            if site.file.as_path() == root || !site_replaced(site, &chain) {
                continue;
            }
            suppressed.insert((site.file.clone(), site.offset));
        }
        suppressed
    }
}

fn site_replaced(site: &ResolvedCallSite, chain: &Chain<'_>) -> bool {
    if site.has_unknown_target() {
        return false;
    }
    let mut matched = false;
    for candidate in site.targets() {
        matched = true;
        if chain_hits(site.file.as_path(), candidate.target, chain) != Some(true) {
            return false;
        }
    }
    matched
}

fn chain_hits(from: &Path, target: KnownCallTargetRef<'_>, chain: &Chain<'_>) -> Option<bool> {
    match target {
        KnownCallTargetRef::ModuleExport {
            specifier,
            export_path,
            ..
        } => follow(chain, from, specifier, export_path, &mut fx_set()),
        KnownCallTargetRef::RepositoryFunction { file, .. } => Some(
            chain
                .modules
                .contains(&MockedModule::File(file.to_path_buf())),
        ),
        KnownCallTargetRef::Global { .. } => Some(false),
    }
}

fn follow(
    chain: &Chain<'_>,
    from: &Path,
    specifier: &str,
    export: &str,
    seen: &mut ExportSeen,
) -> Option<bool> {
    if export.contains('.') {
        return None;
    }
    let id = module_id(chain, specifier, from);
    if chain.modules.contains(&id) {
        return Some(true);
    }
    let MockedModule::File(path) = id else {
        return Some(false);
    };
    if !seen.insert((path.clone(), export.to_string())) {
        return Some(false);
    }
    let hop = {
        let facts = chain.facts.get_ts_facts(&path)?;
        if facts.parse_error.is_some() || facts.operational_error.is_some() {
            return None;
        }
        owned_hop(facts, export)
    };
    match hop {
        Hop::Uncertain => None,
        Hop::Miss => Some(false),
        Hop::Next(specifier, name) => follow(chain, &path, &specifier, &name, seen),
        Hop::Stars(stars) => {
            for specifier in &stars {
                let mut branch = seen.clone();
                let hop = follow(chain, &path, specifier, export, &mut branch);
                if hop != Some(true) {
                    return hop;
                }
            }
            Some(true)
        }
    }
}

fn module_id(chain: &Chain<'_>, specifier: &str, from: &Path) -> MockedModule {
    match chain.resolver.resolve(specifier, from) {
        Some(path) => chain
            .files
            .visible_path(&path)
            .map(|visible| MockedModule::File(visible.to_path_buf()))
            .unwrap_or_else(|| {
                MockedModule::File(crate::codebase::ts_resolver::normalize_path(&path))
            }),
        None => MockedModule::Specifier(specifier.to_string()),
    }
}
