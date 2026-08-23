use super::*;
use oxc_allocator::Allocator;
use oxc_span::SourceType;
use std::path::Path;

mod basic;
mod coverage;
mod review;
mod semantics;

fn facts(source: &str) -> ResourceFacts {
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(
        Path::new("resource.ts"),
        &allocator,
        source,
        SourceType::ts(),
    );
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    extract(&parsed.program, source)
}
