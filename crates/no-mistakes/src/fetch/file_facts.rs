use crate::fetch::imports::{
    collect_identifier_references, collect_imports_from_program_from_visible,
    collect_runtime_imports_from_program_from_visible,
};
use crate::fetch::resolve::relative_string;
use crate::fetch::types::FetchOccurrence;
use crate::fetch::visitor::FetchVisitor;
use anyhow::Result;
use oxc_ast_visit::Visit;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) struct ParsedFileFacts {
    pub(crate) has_use_client_directive: bool,
    pub(crate) has_use_server_directive: bool,
    pub(crate) fetches: Vec<FetchOccurrence>,
    pub(crate) imports: Vec<PathBuf>,
    pub(crate) used_imports: Vec<PathBuf>,
}

#[derive(Default)]
pub struct ParsedFileCache {
    files: HashMap<PathBuf, Result<ParsedFileFacts, String>>,
}

impl ParsedFileCache {
    pub(crate) fn load(
        &mut self,
        path: &Path,
        root: &Path,
        import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
        visible_files: &HashSet<PathBuf>,
    ) -> Result<ParsedFileFacts> {
        let abs_path = crate::codebase::ts_resolver::normalize_path(path);
        if let Some(facts) = self.files.get(&abs_path) {
            return facts.clone().map_err(anyhow::Error::msg);
        }

        let result = (|| {
            let source = std::fs::read_to_string(&abs_path)?;
            crate::ast::with_program(&abs_path, &source, |program, _| {
                ParsedFileFacts::from_program(
                    &abs_path,
                    root,
                    &source,
                    program,
                    import_cache,
                    visible_files,
                )
            })
        })();
        let cached = result.map_err(|error: anyhow::Error| format!("{error:#}"));
        self.files.insert(abs_path, cached.clone());
        cached.map_err(anyhow::Error::msg)
    }

    pub(crate) fn insert(&mut self, path: PathBuf, facts: ParsedFileFacts) {
        self.files.insert(path, Ok(facts));
    }

    pub(crate) fn insert_error(&mut self, path: PathBuf, error: String) {
        self.files.insert(path, Err(error));
    }
}

impl ParsedFileFacts {
    pub(crate) fn from_program(
        path: &Path,
        root: &Path,
        source: &str,
        program: &oxc_ast::ast::Program<'_>,
        import_cache: &mut HashMap<PathBuf, Vec<PathBuf>>,
        visible_files: &HashSet<PathBuf>,
    ) -> Self {
        let referenced_identifiers = collect_identifier_references(program);
        let used_imports = collect_runtime_imports_from_program_from_visible(
            path,
            program,
            &referenced_identifiers,
            visible_files,
        );
        let imports =
            collect_imports_from_program_from_visible(path, program, import_cache, visible_files);
        let mut visitor = FetchVisitor::new(source, &relative_string(root, path), false, false);
        visitor.visit_program(program);
        Self {
            has_use_client_directive: program
                .directives
                .iter()
                .any(|directive| directive.directive == "use client"),
            has_use_server_directive: program
                .directives
                .iter()
                .any(|directive| directive.directive == "use server"),
            fetches: visitor.fetches,
            imports,
            used_imports,
        }
    }
}
