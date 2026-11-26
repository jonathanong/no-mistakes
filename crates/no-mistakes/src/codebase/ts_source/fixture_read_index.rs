use super::fixture_reads::FixtureReads;
use dashmap::DashMap;
use globset::GlobSet;
use oxc_ast::ast::Program;
use regex::RegexSet;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[doc(hidden)]
pub struct FixtureReadScope {
    tests: GlobSet,
    literals: Vec<String>,
    matcher: RegexSet,
}

impl FixtureReadScope {
    pub fn new(tests: GlobSet, literals: Vec<String>) -> Result<Self, regex::Error> {
        let matcher = RegexSet::new(literals.iter().map(|literal| regex::escape(literal)))?;
        Ok(Self {
            tests,
            literals,
            matcher,
        })
    }
}

/// File-local fixture observations owned by one prepared test-plan request.
#[doc(hidden)]
pub struct FixtureReadIndex {
    root: PathBuf,
    scopes: Vec<FixtureReadScope>,
    reads: DashMap<PathBuf, FixtureReads>,
}

impl std::fmt::Debug for FixtureReadIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FixtureReadIndex")
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

impl FixtureReadIndex {
    pub fn new(root: PathBuf, scopes: Vec<FixtureReadScope>) -> Self {
        Self {
            root,
            scopes,
            reads: DashMap::new(),
        }
    }

    pub(crate) fn record(
        &self,
        path: &Path,
        source: &str,
        program: &Program<'_>,
        semantic: Option<&oxc_semantic::SemanticBuilderReturn<'_>>,
    ) {
        let literals = self.literals_for(path, source);
        if literals.is_empty() {
            return;
        }
        self.reads
            .entry(path.to_path_buf())
            .or_insert_with(|| match semantic {
                Some(built) if built.diagnostics.is_empty() => {
                    FixtureReads::from_semantic(&built.semantic, source, &literals)
                }
                Some(_) => FixtureReads::default(),
                None => FixtureReads::from_program(program, source, &literals),
            });
    }

    pub fn affects(
        &self,
        path: &Path,
        source: &str,
        literal: &str,
        attributes: &[String],
        allow_parse: bool,
    ) -> bool {
        if let Some(reads) = self.reads.get(path) {
            return reads.affects(source, literal, attributes);
        }
        if !allow_parse {
            return true;
        }
        let reads = self.reads.entry(path.to_path_buf()).or_insert_with(|| {
            let literals = self.literals_for(path, source);
            if literals.is_empty() {
                FixtureReads::default()
            } else {
                FixtureReads::collect(path, source, &literals)
            }
        });
        reads.affects(source, literal, attributes)
    }

    fn literals_for(&self, path: &Path, source: &str) -> BTreeSet<String> {
        let Ok(relative) = path.strip_prefix(&self.root) else {
            return BTreeSet::new();
        };
        self.scopes
            .iter()
            .filter(|scope| scope.tests.is_match(relative))
            .flat_map(|scope| {
                scope
                    .matcher
                    .matches(source)
                    .into_iter()
                    .map(|index| scope.literals[index].clone())
            })
            .collect()
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;
