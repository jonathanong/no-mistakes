use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_span::SourceType;
use self_cell::self_cell;
use std::cell::RefCell;
use std::path::Path;
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

type CachedPrograms = FxHashMap<(Arc<Path>, ParseMode), Result<Rc<CachedProgram>, String>>;

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
    interned_paths: Rc<RefCell<FxHashMap<Arc<Path>, ()>>>,
    entries: Rc<RefCell<CachedPrograms>>,
}

#[cfg(test)]
pub(super) mod tests;

mod intern;
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
        let cached = self.cached_program(path, source, ParseMode::Standard, on_parse)?;
        cached.with_dependent(|owner, parsed| match &parsed.strict_error {
            Some(error) => Err(error.clone()),
            None => Ok(analyze(&parsed.program, owner.source.as_ref())),
        })
    }

    pub(crate) fn clear(&self) {
        self.entries.borrow_mut().clear();
        self.interned_paths.borrow_mut().clear();
    }

    pub(crate) fn remove_path(&self, path: &Path) {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let Some((path, ())) = self
            .interned_paths
            .borrow_mut()
            .remove_entry(path.as_path())
        else {
            return;
        };
        let mut entries = self.entries.borrow_mut();
        for mode in [
            ParseMode::Standard,
            ParseMode::TypeScriptFallback,
            ParseMode::LegacySymbols,
        ] {
            entries.remove(&(Arc::clone(&path), mode));
        }
    }

    pub(crate) fn entry_count(&self) -> usize {
        self.entries.borrow().len()
    }

    /// Whether the preparing thread already parsed this native source mode,
    /// including a recovered program or cached parser failure.
    pub(crate) fn contains_path(&self, path: &Path) -> bool {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        self.interned_lookup(&path).is_some_and(|path| {
            self.entries
                .borrow()
                .contains_key(&(path, ParseMode::Standard))
        })
    }

    pub(crate) fn parse_error(&self, path: &Path) -> Option<String> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let interned = self.interned_lookup(&path)?;
        let cached = self
            .entries
            .borrow()
            .get(&(interned, ParseMode::Standard))?
            .clone();
        match cached {
            Ok(cached) => cached.with_dependent(|_, parsed| parsed.strict_error.clone()),
            Err(error) => Some(error),
        }
    }

    fn cached_program(
        &self,
        path: &Path,
        source: Arc<str>,
        mode: ParseMode,
        on_parse: impl FnOnce(),
    ) -> Result<Rc<CachedProgram>, String> {
        let path = self.intern_path(crate::codebase::ts_resolver::normalize_path(path));
        let key = (Arc::clone(&path), mode);
        if let Some(cached) = self.entries.borrow().get(&key) {
            return cached.clone();
        }
        on_parse();
        let cached = parse_program(&path, source, mode).map(Rc::new);
        self.entries.borrow_mut().insert(key, cached.clone());
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

#[cfg(test)]
mod eviction_tests;
