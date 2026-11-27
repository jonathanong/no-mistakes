use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_span::SourceType;
use std::path::Path;

use crate::codebase::testing_analysis::dynamic_imports::Collector;
pub use crate::codebase::testing_analysis::dynamic_imports::{DynamicImport, TestFacts};

pub fn extract(path: &Path, source: &str) -> Result<TestFacts> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).context(format!(
        "unsupported JavaScript/TypeScript file: {}",
        path.display()
    ))?;
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    Ok(extract_program(source, &parsed.program))
}

fn extract_program(source: &str, program: &Program<'_>) -> TestFacts {
    let mut visitor = Collector::new(source, program);
    visitor.visit_program(program);
    visitor.into_facts()
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
