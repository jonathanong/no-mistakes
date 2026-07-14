use anyhow::Context;
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
    parse_error: Option<String>,
}

type CachedPrograms = HashMap<PathBuf, Result<Rc<CachedProgram>, String>>;

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

impl ParsedProgramCache {
    pub(crate) fn with_program<T>(
        &self,
        path: &Path,
        source: &str,
        analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
    ) -> Result<T, String> {
        let cached = self.cached_program(path, source)?;
        cached.with_dependent(|owner, parsed| match &parsed.parse_error {
            Some(error) => Err(error.clone()),
            None => Ok(analyze(&parsed.program, owner.source.as_str())),
        })
    }

    pub(crate) fn clear(&self) {
        self.entries.borrow_mut().clear();
    }

    pub(crate) fn parse_error(&self, path: &Path) -> Option<String> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let cached = self.entries.borrow().get(&path)?.clone();
        match cached {
            Ok(cached) => cached.with_dependent(|_, parsed| parsed.parse_error.clone()),
            Err(error) => Some(error),
        }
    }

    fn cached_program(&self, path: &Path, source: &str) -> Result<Rc<CachedProgram>, String> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        if let Some(cached) = self.entries.borrow().get(&path) {
            return cached.clone();
        }
        let cached = parse_program(&path, source).map(Rc::new);
        self.entries.borrow_mut().insert(path, cached.clone());
        cached
    }
}

fn parse_program(path: &Path, source: &str) -> Result<CachedProgram, String> {
    let source_type = SourceType::from_path(path)
        .with_context(|| format!("unsupported JavaScript/TypeScript file: {}", path.display()))
        .map_err(|error| error.to_string())?;
    let owner = ProgramOwner {
        allocator: Allocator::default(),
        source: source.to_string(),
        source_type,
    };
    CachedProgram::try_new(owner, |owner| {
        let parsed = crate::ast::parse(path, &owner.allocator, &owner.source, owner.source_type);
        let parse_error = if parsed.panicked || !parsed.diagnostics.is_empty() {
            let detail = parsed
                .diagnostics
                .first()
                .map(|error| format!("{error:?}"))
                .unwrap_or("unknown error (parser panicked)".to_string());
            Some(detail)
        } else {
            None
        };
        Ok(ParsedProgram {
            program: parsed.program,
            parse_error,
        })
    })
}
