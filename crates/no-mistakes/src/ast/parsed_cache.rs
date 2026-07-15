use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_span::SourceType;
use self_cell::self_cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

struct ProgramOwner {
    allocator: Allocator,
    source: String,
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

type CachedPrograms = HashMap<(PathBuf, ParseMode), Result<Rc<CachedProgram>, String>>;

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

mod parse;
use parse::parse_program;

impl ParsedProgramCache {
    pub(crate) fn with_program<T>(
        &self,
        path: &Path,
        source: &str,
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
    ) -> Result<T, String> {
        self.with_program_observed(path, source, || {}, analyze)
    }

    pub(crate) fn with_program_observed<T>(
        &self,
        path: &Path,
        source: &str,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
    ) -> Result<T, String> {
        let cached = self.cached_program(path, source, ParseMode::Standard, on_parse)?;
        cached.with_dependent(|owner, parsed| match &parsed.strict_error {
            Some(error) => Err(error.clone()),
            None => Ok(analyze(&parsed.program, owner.source.as_str())),
        })
    }

    pub(crate) fn clear(&self) {
        self.entries.borrow_mut().clear();
    }

    pub(crate) fn parse_error(&self, path: &Path) -> Option<String> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let cached = self
            .entries
            .borrow()
            .get(&(path, ParseMode::Standard))?
            .clone();
        match cached {
            Ok(cached) => cached.with_dependent(|_, parsed| parsed.strict_error.clone()),
            Err(error) => Some(error),
        }
    }

    pub(crate) fn with_recovered_program_observed<T>(
        &self,
        path: &Path,
        source: &str,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
    ) -> Result<T, String> {
        let cached = self.cached_program(path, source, ParseMode::Standard, on_parse)?;
        Ok(cached.with_dependent(|owner, parsed| {
            analyze(
                &parsed.program,
                owner.source.as_str(),
                parsed.diagnostic_error.clone(),
            )
        }))
    }

    pub(crate) fn with_recovered_typescript_program_observed<T>(
        &self,
        path: &Path,
        source: &str,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
    ) -> Result<T, String> {
        let cached = self.cached_program(path, source, ParseMode::TypeScriptFallback, on_parse)?;
        Ok(cached.with_dependent(|owner, parsed| {
            analyze(
                &parsed.program,
                owner.source.as_str(),
                parsed.diagnostic_error.clone(),
            )
        }))
    }

    pub(crate) fn with_legacy_symbols_program_observed<T>(
        &self,
        path: &Path,
        source: &str,
        on_parse: impl FnOnce(),
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
    ) -> Result<T, String> {
        let mode = if legacy_symbols_share_standard_parse(path) {
            ParseMode::Standard
        } else {
            ParseMode::LegacySymbols
        };
        let cached = self.cached_program(path, source, mode, on_parse)?;
        cached.with_dependent(|owner, parsed| match &parsed.panic_error {
            Some(error) => Err(error.clone()),
            None => Ok(analyze(
                &parsed.program,
                owner.source.as_str(),
                parsed.diagnostic_error.clone(),
            )),
        })
    }

    fn cached_program(
        &self,
        path: &Path,
        source: &str,
        mode: ParseMode,
        on_parse: impl FnOnce(),
    ) -> Result<Rc<CachedProgram>, String> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let key = (path.clone(), mode);
        if let Some(cached) = self.entries.borrow().get(&key) {
            return cached.clone();
        }
        on_parse();
        let cached = parse_program(&path, source, mode).map(Rc::new);
        self.entries.borrow_mut().insert(key, cached.clone());
        cached
    }
}

pub(super) fn legacy_symbols_share_standard_parse(path: &Path) -> bool {
    let Ok(source_type) = SourceType::from_path(path) else {
        return false;
    };
    !source_type.is_javascript()
        && !source_type.is_typescript_definition()
        && source_type.is_unambiguous()
}
