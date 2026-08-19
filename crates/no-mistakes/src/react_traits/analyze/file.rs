use crate::ast;
use crate::imports::{
    collect_identifier_references, collect_runtime_imports_from_program, relative_string,
};
use crate::react_traits::analyze::components::extract_components;
use crate::react_traits::analyze::environment::{detect_file_environment, FileEnvironment};
use crate::react_traits::analyze::import_table::{
    build_import_table, build_import_table_from_visible,
};
use crate::react_traits::report::types::{ComponentFacts, ComponentRef, Environment, FetchCall};
use crate::react_traits::traits;
use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(crate) struct FileAnalysis {
    pub(crate) components: std::sync::Arc<Vec<ComponentFacts>>,
}

#[cfg(test)]
mod tests;

pub(crate) fn analyze_file(abs_path: &Path, root: &Path) -> Result<FileAnalysis> {
    analyze_file_inner(abs_path, root, None)
}

pub(crate) fn analyze_file_from_visible(
    abs_path: &Path,
    root: &Path,
    visible_files: &HashSet<PathBuf>,
) -> Result<FileAnalysis> {
    analyze_file_inner(abs_path, root, Some(visible_files))
}

fn analyze_file_inner(
    abs_path: &Path,
    root: &Path,
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<FileAnalysis> {
    let source = std::fs::read_to_string(abs_path)?;
    ast::with_program(abs_path, &source, |program, _src| {
        analyze_program_inner(abs_path, root, &source, program, visible_files)
    })
}

pub(crate) fn analyze_program(
    abs_path: &Path,
    root: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> FileAnalysis {
    analyze_program_inner(abs_path, root, source, program, None)
}

pub(crate) fn analyze_program_from_visible(
    abs_path: &Path,
    root: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    visible_files: &HashSet<PathBuf>,
) -> FileAnalysis {
    analyze_program_inner(abs_path, root, source, program, Some(visible_files))
}

fn analyze_program_inner(
    abs_path: &Path,
    root: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
    visible_files: Option<&HashSet<PathBuf>>,
) -> FileAnalysis {
    let rel_path = relative_string(root, abs_path);

    let components = {
        let env = detect_file_environment(program);
        let import_table = match visible_files {
            Some(visible) => build_import_table_from_visible(abs_path, program, visible),
            None => build_import_table(abs_path, program),
        };
        let referenced = collect_identifier_references(program);
        let deps = match visible_files {
            Some(visible) => {
                crate::fetch::imports::collect_runtime_imports_from_program_from_visible(
                    abs_path,
                    program,
                    &referenced,
                    visible,
                )
            }
            None => collect_runtime_imports_from_program(abs_path, program, &referenced),
        };
        let environment = match env {
            FileEnvironment::Server => Environment::Server,
            FileEnvironment::Client => Environment::Client,
            FileEnvironment::Unknown => Environment::Unknown,
        };
        let dep_strings: Vec<String> = deps.iter().map(|p| relative_string(root, p)).collect();
        let component_defs = extract_components(program);
        let spans: Vec<oxc_span::Span> = component_defs.iter().map(|def| def.span).collect();
        let dynamic_names = traits::suspense::collect_dynamic_names_for_spans(program, &spans);
        let hits = traits::file_walk::collect_file_trait_hits(
            program,
            &spans,
            &dynamic_names,
            &import_table,
            abs_path,
        );
        let fetches = traits::fetch::collect_fetch_calls_in_file(program, source, &rel_path);

        let mut components = Vec::new();
        for (i, def) in component_defs.into_iter().enumerate() {
            let has_state = hits.has_state[i];
            let has_props = traits::props::has_function_params(program, def.span);
            let passes_props = hits.passes_props[i];
            let uses_memo = hits.uses_memo[i] || traits::memo::is_wrapped_in_memo(program, &def);
            let uses_context_provider = hits.uses_context_provider[i];
            let uses_suspense = traits::suspense::is_component_direct_lazy(program, def.span)
                || hits.uses_suspense_jsx[i];
            let component_fetches = fetches
                .iter()
                .filter(|(span, _)| span.start >= def.span.start && span.end <= def.span.end)
                .map(|(_, f)| FetchCall {
                    file: f.file.clone(),
                    exported_name: f.cached_function.clone(),
                    shape: Some(format!("{} {}", f.method, f.path)),
                    line: f.line,
                })
                .collect();
            let children: Vec<ComponentRef> = hits.children[i]
                .iter()
                .map(|(path, name)| ComponentRef {
                    name: name.clone(),
                    file: relative_string(root, path),
                })
                .collect();

            components.push(ComponentFacts {
                name: def.name.clone(),
                file: rel_path.clone(),
                environment: environment.clone(),
                has_state,
                has_props,
                passes_props,
                uses_memo,
                uses_context_provider,
                uses_suspense,
                fetches: component_fetches,
                dependencies: dep_strings.clone(),
                children,
                inherited_from_children: None,
            });
        }

        components
    };

    FileAnalysis {
        components: components.into(),
    }
}
