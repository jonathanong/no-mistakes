use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_span::SourceType;
use self_cell::self_cell;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use crate::fx::FxHashMap;

struct ProgramOwner {
    allocator: Allocator,
    source: Arc<str>,
    source_type: SourceType,
}

struct ParsedProgram<'a> {
    program: Program<'a>,
    strict_error: Option<String>,
    diagnostic_error: Option<String>,
    panic_error: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ParseMode {
    Standard,
    TypeScriptFallback,
    LegacySymbols,
}

// One owned path and one cache handle cover every supported parse mode.
// Removing the path drops its whole bucket without scanning unrelated entries.
type CachedModes = [Option<Rc<Result<CachedProgram, String>>>; 3];

#[derive(Default)]
struct CachedPrograms {
    paths: FxHashMap<PathBuf, CachedModes>,
    count: usize,
}

self_cell! {
    struct CachedProgram {
        owner: ProgramOwner,

        #[covariant]
        dependent: ParsedProgram,
    }
}

/// A same-thread, request-scoped parser cache. OXC programs never cross a
/// thread boundary; only owned facts derived from them may leave the scope.
#[derive(Clone, Default)]
pub(crate) struct ParsedProgramCache {
    entries: Rc<RefCell<CachedPrograms>>,
}

#[cfg(test)]
pub(super) mod tests;

#[cfg(not(test))]
macro_rules! eviction_probe {
    () => {};
}

#[cfg(test)]
#[macro_use]
mod eviction_tests;

mod modes;
mod parse;
use parse::parse_program;

impl ParsedProgramCache {
    pub(crate) fn with_program<T>(
        &self,
        path: &Path,
        source: Arc<str>,
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
    ) -> Result<T, String> {
        self.with_program_observed(path, source, || {}, analyze)
    }

    pub(crate) fn with_program_observed<T>(
        &self,
        path: &Path,
        source: Arc<str>,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
    ) -> Result<T, String> {
        self.with_cached_program(path, source, ParseMode::Standard, on_parse, |cached| {
            cached.with_dependent(|owner, parsed| match &parsed.strict_error {
                Some(error) => Err(error.clone()),
                None => Ok(analyze(&parsed.program, owner.source.as_ref())),
            })
        })
    }

    pub(crate) fn clear(&self) {
        let mut entries = self.entries.borrow_mut();
        entries.paths.clear();
        entries.count = 0;
    }

    pub(crate) fn remove_path(&self, path: &Path) {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let mut entries = self.entries.borrow_mut();
        eviction_probe!();
        if let Some(modes) = entries.paths.remove(path.as_path()) {
            entries.count -= modes.iter().flatten().count();
        }
    }

    pub(crate) fn entry_count(&self) -> usize {
        self.entries.borrow().count
    }

    /// Whether the preparing thread already parsed this native source mode,
    /// including a recovered program or cached parser failure.
    pub(crate) fn contains_path(&self, path: &Path) -> bool {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        self.entries
            .borrow()
            .paths
            .get(path.as_path())
            .is_some_and(|modes| modes[ParseMode::Standard as usize].is_some())
    }

    pub(crate) fn parse_error(&self, path: &Path) -> Option<String> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let cached = self.entries.borrow().paths.get(path.as_path())?[ParseMode::Standard as usize]
            .clone()?;
        match cached.as_ref() {
            Ok(cached) => cached.with_dependent(|_, parsed| parsed.strict_error.clone()),
            Err(error) => Some(error.clone()),
        }
    }

    fn with_cached_program<T>(
        &self,
        path: &Path,
        source: Arc<str>,
        mode: ParseMode,
        on_parse: impl FnOnce(),
        analyze: impl FnOnce(&CachedProgram) -> Result<T, String>,
    ) -> Result<T, String> {
        let outcome = self.cached_program(path, source, mode, on_parse);
        match outcome.as_ref() {
            Ok(cached) => analyze(cached),
            Err(error) => Err(error.clone()),
        }
    }

    fn cached_program(
        &self,
        path: &Path,
        source: Arc<str>,
        mode: ParseMode,
        on_parse: impl FnOnce(),
    ) -> Rc<Result<CachedProgram, String>> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if let Some(cached) = self
            .entries
            .borrow()
            .paths
            .get(path.as_path())
            .and_then(|modes| modes[mode as usize].as_ref())
        {
            return Rc::clone(cached);
        }
        on_parse();
        let cached = Rc::new(parse_program(&path, source, mode));
        let mut entries = self.entries.borrow_mut();
        let previous =
            entries.paths.entry(path).or_default()[mode as usize].replace(Rc::clone(&cached));
        entries.count += usize::from(previous.is_none());
        cached
    }
}

/// Whether the regular file-backed parser uses exactly the legacy
/// `extract_symbols_at_path` source type for `path`.
pub(crate) fn legacy_symbols_share_standard_parse(path: &Path) -> bool {
    let Ok(source_type) = SourceType::from_path(path) else {
        return false;
    };
    !source_type.is_javascript()
        && !source_type.is_typescript_definition()
        && source_type.is_unambiguous()
}
