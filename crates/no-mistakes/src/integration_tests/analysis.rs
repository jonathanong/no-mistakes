use super::calls;
use super::types::{CallTarget, FileAnalysis, FunctionInfo, FunctionKey, TestCase};
use crate::ast;
use anyhow::{Context, Result};
use oxc_ast::ast::{BindingPattern, ModuleExportName, Program};
use oxc_ast_visit::Visit;
use oxc_span::Span;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod analysis_helpers;
#[cfg(not(test))]
macro_rules! record_discovery_visit {
    () => {};
}
#[cfg(test)]
#[macro_use]
mod tests;
mod visitor;

pub(super) fn analyze_files_with_seed(
    files: &[PathBuf],
    mut analyses: BTreeMap<PathBuf, FileAnalysis>,
) -> Result<BTreeMap<PathBuf, FileAnalysis>> {
    for file in files {
        if analyses.contains_key(file) {
            continue;
        }
        let source = match std::fs::read_to_string(file) {
            Ok(source) => source,
            Err(_) => continue,
        };
        let analysis = ast::with_program(file, &source, |program, source| {
            analyze_program(file, program, source)
        })
        .with_context(|| format!("analyzing integration annotations in {}", file.display()))?;
        analyses.insert(file.clone(), analysis);
    }
    Ok(analyses)
}

pub(crate) fn analyze_program(path: &Path, program: &Program<'_>, source: &str) -> FileAnalysis {
    analyze_program_with_test_discovery(path, program, source, true)
}

pub(crate) fn analyze_program_with_test_discovery(
    path: &Path,
    program: &Program<'_>,
    source: &str,
    collect_tests: bool,
) -> FileAnalysis {
    let collect_tests = collect_tests || source_may_contain_nested_imports(source);
    let mut collector = AnalysisCollector {
        path,
        source,
        result: FileAnalysis::default(),
        describe_stack: Vec::new(),
        statement_depth: 0,
        test_counter: 0,
        collect_tests,
    };
    collector.visit_program(program);
    // Synthetic test functions share the historical helper namespace. Keep
    // discovery for nested import containers and helpers that can reference or
    // collide with __test_N.
    if !collect_tests
        && (collector
            .result
            .functions
            .keys()
            .any(|name| name.starts_with("__test_"))
            || collector
                .result
                .exports
                .values()
                .any(|name| name.starts_with("__test_"))
            || collector.result.functions.values().any(|function| {
                function.calls.iter().any(|call| match call {
                    CallTarget::Local(name) => name.starts_with("__test_"),
                    _ => false,
                })
            }))
    {
        return analyze_program(path, program, source);
    }
    collector.result
}

// Ambient modules/namespaces can update the historical file-wide import table
// at any nesting depth. Escapes conservatively cover contextual keyword lexer
// variation; false positives retain full discovery without another AST pass.
fn source_may_contain_nested_imports(source: &str) -> bool {
    source.contains('\\')
        || ["module", "namespace"].into_iter().any(|keyword| {
            source.match_indices(keyword).any(|(start, _)| {
                let end = start + keyword.len();
                let identifier =
                    |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
                (start == 0 || !identifier(source.as_bytes()[start - 1]))
                    && (end == source.len() || !identifier(source.as_bytes()[end]))
            })
        })
}

struct AnalysisCollector<'a, 'p> {
    path: &'p Path,
    source: &'a str,
    result: FileAnalysis,
    describe_stack: Vec<String>,
    statement_depth: usize,
    test_counter: usize,
    collect_tests: bool,
}

impl AnalysisCollector<'_, '_> {
    fn collect_test(
        &mut self,
        call_start: u32,
        test_name: String,
        argument: &oxc_ast::ast::Argument<'_>,
        span: Span,
    ) {
        self.test_counter += 1;
        let function_name = format!("__test_{}", self.test_counter);
        self.result.functions.insert(
            function_name.clone(),
            FunctionInfo {
                integration: calls::integration_annotation_before(self.source, span),
                calls: calls::collect_calls(argument),
            },
        );
        self.result.tests.push(TestCase {
            name: Some(test_name),
            describe_path: self.describe_stack.clone(),
            function_key: FunctionKey {
                file: self.path.to_path_buf(),
                name: function_name,
            },
            line: crate::codebase::ts_source::byte_offset_to_line(self.source, call_start as usize),
        });
    }
}

fn binding_name<'a>(pattern: &'a BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn module_export_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(id) => id.name.to_string(),
        ModuleExportName::IdentifierReference(id) => id.name.to_string(),
        ModuleExportName::StringLiteral(value) => value.value.to_string(),
    }
}
