use anyhow::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use std::cell::RefCell;
use std::path::Path;

mod expression;
mod parsed_cache;
pub use expression::{binary_concat_path_text, expression_path, span_text, template_literal_text};
pub(crate) use parsed_cache::ParsedProgramCache;

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

pub(crate) fn request_parse_cache_active() -> bool {
    REQUEST_PARSE_CACHES.with(|caches| !caches.borrow().is_empty())
}

pub(crate) fn clear_request_parse_cache() {
    if let Some(cache) = current_request_parse_cache() {
        cache.clear();
    }
}

#[doc(hidden)]
pub fn with_request_parse_cache<T>(collect: impl FnOnce() -> T) -> T {
    let cache = current_request_parse_cache().unwrap_or_default();
    REQUEST_PARSE_CACHES.with(|caches| caches.borrow_mut().push(cache));
    let _guard = RequestParseCacheGuard;
    collect()
}

#[cfg(any(test, feature = "test-instrumentation"))]
struct ParseCountSession {
    owner: std::thread::ThreadId,
    counts: std::collections::HashMap<std::path::PathBuf, usize>,
}

#[cfg(any(test, feature = "test-instrumentation"))]
type ParseCounts = std::collections::HashMap<std::path::PathBuf, ParseCountSession>;

#[cfg(any(test, feature = "test-instrumentation"))]
fn parse_counts() -> &'static std::sync::Mutex<ParseCounts> {
    static COUNTS: std::sync::OnceLock<std::sync::Mutex<ParseCounts>> = std::sync::OnceLock::new();
    COUNTS.get_or_init(|| std::sync::Mutex::new(ParseCounts::new()))
}

#[doc(hidden)]
#[cfg(any(test, feature = "test-instrumentation"))]
pub fn begin_parse_count(root: &Path) {
    parse_counts()
        .lock()
        .expect("parse-count mutex poisoned")
        .insert(
            root.to_path_buf(),
            ParseCountSession {
                owner: std::thread::current().id(),
                counts: std::collections::HashMap::new(),
            },
        );
}

#[doc(hidden)]
#[cfg(any(test, feature = "test-instrumentation"))]
pub fn finish_parse_count(root: &Path) -> std::collections::HashMap<std::path::PathBuf, usize> {
    parse_counts()
        .lock()
        .expect("parse-count mutex poisoned")
        .remove(root)
        .map(|session| session.counts)
        .unwrap_or_default()
}

#[cfg(any(test, feature = "test-instrumentation"))]
pub(crate) fn record_parse_path(path: &Path) {
    let mut counts = parse_counts().lock().expect("parse-count mutex poisoned");
    let current_thread = std::thread::current().id();
    for (root, session) in counts.iter_mut() {
        // Synthetic parses conventionally use relative sentinel paths and may run on a
        // worker rather than the thread that opened the request observation. Only the owning
        // thread may attribute relative sentinels; observed worker parses must use paths rooted
        // in their request so parallel sessions cannot contaminate one another.
        let owns_relative_parse = path.is_relative() && session.owner == current_thread;
        if path.starts_with(root) || owns_relative_parse {
            *session.counts.entry(path.to_path_buf()).or_insert(0) += 1;
        }
    }
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
    record_parse_path(path);
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

/// Parse a JavaScript or TypeScript source while preserving OXC's recovered
/// program when diagnostics are present. `on_parse` runs only for a physical
/// parser invocation, not for a request-cache hit.
pub(crate) fn with_recovered_program_observed<T>(
    path: &Path,
    source: &str,
    on_parse: impl FnOnce(),
    analyze: impl for<'a> FnOnce(&'a Program<'a>, &'a str, Option<String>) -> T,
) -> Result<T> {
    let cache = current_request_parse_cache().unwrap_or_default();
    cache
        .with_recovered_program_observed(path, source, on_parse, analyze)
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
