use crate::codebase::ts_source::SourceStore;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

mod expression;
mod module;
mod mutations;

#[derive(Clone, Debug)]
pub(crate) struct Value {
    pub(crate) kind: Kind,
    pub(crate) file: PathBuf,
    pub(crate) line: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Literal(serde_json::Value),
    Object(BTreeMap<String, Value>, bool),
    Array(Vec<Value>),
    Unknown,
}

impl Value {
    pub(crate) fn property(&self, name: &str) -> Option<Self> {
        match &self.kind {
            Kind::Object(properties, opaque) => properties
                .get(name)
                .cloned()
                .or_else(|| opaque.then(|| self.unknown())),
            _ => Some(self.unknown()),
        }
    }
    pub(crate) fn unknown(&self) -> Self {
        Self {
            kind: Kind::Unknown,
            ..self.clone()
        }
    }
    pub(crate) fn literal(&self) -> Option<&serde_json::Value> {
        match &self.kind {
            Kind::Literal(value) => Some(value),
            _ => None,
        }
    }
}

pub(crate) struct Reader<'a> {
    sources: &'a SourceStore,
    allowed: BTreeSet<PathBuf>,
    modules: BTreeMap<PathBuf, Module>,
    visiting: BTreeSet<PathBuf>,
    programs: crate::ast::ParsedProgramCache,
}

impl<'a> Reader<'a> {
    pub(crate) fn new(sources: &'a SourceStore, paths: BTreeSet<PathBuf>) -> Self {
        Self {
            sources,
            allowed: paths,
            modules: BTreeMap::new(),
            visiting: BTreeSet::new(),
            programs: crate::ast::current_request_parse_cache().unwrap_or_default(),
        }
    }
    pub(crate) fn read(&mut self, path: &Path) -> Value {
        self.module(path)
            .exports
            .get("default")
            .cloned()
            .unwrap_or(Value {
                kind: Kind::Unknown,
                file: path.to_path_buf(),
                line: 1,
            })
    }
    fn module(&mut self, path: &Path) -> Module {
        if let Some(module) = self.modules.get(path) {
            return module.clone();
        }
        if !self.allowed.contains(path) || !self.visiting.insert(path.to_path_buf()) {
            return Module::default();
        }
        let result = self
            .sources
            .read_source_path(path)
            .ok()
            .and_then(|source| {
                let programs = self.programs.clone();
                programs
                    .with_program(path, source, |program, source| {
                        module::parse(self, path, source, program)
                    })
                    .ok()
            })
            .unwrap_or_default();
        self.visiting.remove(path);
        self.modules.insert(path.to_path_buf(), result.clone());
        result
    }
    fn import(&mut self, from: &Path, specifier: &str) -> Module {
        if !specifier.starts_with('.') {
            return Module::default();
        }
        let base = crate::codebase::ts_resolver::normalize_path(
            &from.parent().unwrap_or(from).join(specifier),
        );
        let candidate = std::iter::once(base.clone())
            .chain(
                ["ts", "mts", "cts", "js", "mjs", "cjs", "tsx", "jsx"]
                    .into_iter()
                    .flat_map(|extension| {
                        [
                            PathBuf::from(format!("{}.{extension}", base.display())),
                            base.join(format!("index.{extension}")),
                        ]
                    }),
            )
            .find(|candidate| self.allowed.contains(candidate));
        candidate.map(|path| self.module(&path)).unwrap_or_default()
    }
}

#[derive(Clone)]
struct Module {
    exports: BTreeMap<String, Value>,
    opaque: bool,
}
impl Default for Module {
    fn default() -> Self {
        Self {
            exports: BTreeMap::new(),
            opaque: true,
        }
    }
}
impl Module {
    fn namespace(&self, path: &Path, line: usize) -> Value {
        Value {
            kind: Kind::Object(self.exports.clone(), self.opaque),
            file: path.to_path_buf(),
            line,
        }
    }
}
