use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrowFunctionBody, FunctionBody, Program, Statement};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use std::cell::RefCell;
use std::path::Path;

mod expression;
mod parsed_cache;
#[cfg(any(test, feature = "test-instrumentation"))]
mod parse_count;
pub use expression::{binary_concat_path_text, expression_path, span_text, template_literal_text};
pub(crate) use parsed_cache::{legacy_symbols_share_standard_parse, ParsedProgramCache};
#[cfg(any(test, feature = "test-instrumentation"))]
pub use parse_count::{begin_parse_count, finish_parse_count};

thread_local! {
    static REQUEST_PARSE_CACHES: RefCell<Vec<ParsedProgramCache>> = const { RefCell::new(Vec::new()) };
}

struct RequestParseCacheGuard;

impl Drop for RequestParseCacheGuard {
    fn drop(&mut self) {
        REQUEST_PARSE_CACHES.with(|caches| {
            caches
                .borrow_mut()
                .pop()
                .expect("request parse cache must be active");
        });
    }
}

pub(crate) fn current_request_parse_cache() -> Option<ParsedProgramCache> {
    REQUEST_PARSE_CACHES.with(|caches| caches.borrow().last().cloned())
}

/// Oxc 0.143 represents concise arrow bodies as expressions rather than an
/// empty `FunctionBody`. Callers that scan statements must ignore those bodies.
pub(crate) fn arrow_function_body<'a>(
    body: &'a ArrowFunctionBody<'a>,
) -> Option<&'a FunctionBody<'a>> {
    match body {
        ArrowFunctionBody::FunctionBody(body) => Some(body),
        _ => None,
    }
}

pub(crate) fn arrow_function_body_statements<'a>(
    body: &'a ArrowFunctionBody<'a>,
) -> Option<&'a [Statement<'a>]> {
    arrow_function_body(body).map(|body| body.statements.as_slice())
}

pub(crate) fn request_parse_cache_active() -> bool {
    REQUEST_PARSE_CACHES.with(|caches| !caches.borrow().is_empty())
}

/// Drop programs retained by the current request parse cache.
///
/// Public so the CLI binary's check runner can release extract ASTs after
/// fact collection. Library callers already share this crate.
#[doc(hidden)]
pub fn clear_request_parse_cache() {
    if let Some(cache) = current_request_parse_cache() {
        cache.clear();
    }
}

/// Drop every cached parse mode for one path on the current request.
#[doc(hidden)]
pub fn evict_request_parse_cache_path(path: &Path) {
    if let Some(cache) = current_request_parse_cache() {
        cache.remove_path(path);
    }
}

/// Number of programs retained by the current request parse cache.
#[doc(hidden)]
pub fn request_parse_cache_len() -> usize {
    current_request_parse_cache().map_or(0, |cache| cache.entry_count())
}

#[doc(hidden)]
pub fn with_request_parse_cache<T>(collect: impl FnOnce() -> T) -> T {
    let cache = current_request_parse_cache().unwrap_or_default();
    REQUEST_PARSE_CACHES.with(|caches| caches.borrow_mut().push(cache));
    let _guard = RequestParseCacheGuard;
    collect()
}

/// Install a parse cache owned by this call.
///
/// Unlike [`with_request_parse_cache`], this never clones a cache already on
/// the worker. Nested Rayon work on the shared pool can otherwise inherit
/// another request's path-keyed programs.
#[doc(hidden)]
pub fn with_owned_request_parse_cache<T>(collect: impl FnOnce() -> T) -> T {
    REQUEST_PARSE_CACHES.with(|caches| caches.borrow_mut().push(ParsedProgramCache::default()));
    let _guard = RequestParseCacheGuard;
    collect()
}

/// The single production entrypoint for invoking the OXC parser.
///
/// Keeping observation here makes both successful and failed parses visible to
/// request-scoped instrumentation. Source-only compatibility APIs should pass a
/// stable, extension-bearing relative sentinel path.
pub(crate) fn parse<'a>(
    path: &Path,
    allocator: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
) -> ParserReturn<'a> {
    #[cfg(any(test, feature = "test-instrumentation"))]
    parse_count::record_parse_path(path);
    #[cfg(not(any(test, feature = "test-instrumentation")))]
    let _ = path;
    Parser::new(allocator, source, source_type).parse()
}

pub fn with_program<T>(
    path: &Path,
    source: &str,
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
) -> Result<T> {
    if let Some(cache) = current_request_parse_cache() {
        return cache
            .with_program(path, source, analyze)
            .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display()));
    }
    ParsedProgramCache::default()
        .with_program(path, source, analyze)
        .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display()))
}

/// Parse strictly while reporting only a physical parser invocation through
/// `on_parse`.
pub(crate) fn with_program_observed<T>(
    path: &Path,
    source: &str,
    on_parse: impl FnOnce(),
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str) -> T,
) -> Result<T> {
    let cache = current_request_parse_cache().unwrap_or_default();
    cache
        .with_program_observed(path, source, on_parse, analyze)
        .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display()))
}

/// Recovered parse that additionally reports whether the parser panicked.
/// `on_parse` runs only for a physical parser invocation, not for a
/// request-cache hit. General recovered consumers may use partial ASTs; fact
/// collectors can preserve the panic distinction for sound consumers.
pub(crate) fn with_recovered_program_status_observed<T>(
    path: &Path,
    source: &str,
    on_parse: impl FnOnce(),
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>, bool) -> T,
) -> Result<T> {
    let cache = current_request_parse_cache().unwrap_or_default();
    cache
        .with_recovered_program_status_observed(path, source, on_parse, analyze)
        .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display()))
}

/// Parse recovered source with an explicit TypeScript fallback for unknown
/// extensions. `on_parse` has the same physical-work semantics as above.
pub(crate) fn with_recovered_typescript_program_observed<T>(
    path: &Path,
    source: &str,
    on_parse: impl FnOnce(),
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
) -> Result<T> {
    let cache = current_request_parse_cache().unwrap_or_default();
    cache
        .with_recovered_typescript_program_observed(path, source, on_parse, analyze)
        .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display()))
}

/// Parse with the historical symbols source type: TypeScript for every file
/// except `.tsx` and `.jsx`, which use TSX. Recovered diagnostics remain
/// available to the caller; only a parser panic is fatal.
pub(crate) fn with_legacy_symbols_program_observed<T>(
    path: &Path,
    source: &str,
    on_parse: impl FnOnce(),
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
) -> Result<T> {
    let cache = current_request_parse_cache().unwrap_or_default();
    cache
        .with_legacy_symbols_program_observed(path, source, on_parse, analyze)
        .map_err(|detail| anyhow::anyhow!("failed to parse {}: {detail}", path.display()))
}

#[cfg(test)]
pub(crate) mod tests;
