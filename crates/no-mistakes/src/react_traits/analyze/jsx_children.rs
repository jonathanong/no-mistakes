use crate::react_traits::analyze::import_table::ImportTable;
use crate::react_traits::analyze::jsx_resolve::{element_root_and_suffix, resolve_target};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::react_traits::analyze::jsx_resolve::collect_local_components;
use oxc_ast::ast::Program;
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;

#[allow(dead_code)]
struct JsxChildrenVisitor<'a> {
    import_table: &'a ImportTable,
    local_components: &'a HashMap<String, String>,
    file_path: &'a Path,
    span: Span,
    children: Vec<(PathBuf, String)>,
}

impl<'a> JsxChildrenVisitor<'a> {
    fn new(
        import_table: &'a ImportTable,
        local_components: &'a HashMap<String, String>,
        file_path: &'a Path,
        span: Span,
    ) -> Self {
        Self {
            import_table,
            local_components,
            file_path,
            span,
            children: Vec::new(),
        }
    }
}

impl<'a> Visit<'a> for JsxChildrenVisitor<'a> {
    fn visit_jsx_element(&mut self, elem: &oxc_ast::ast::JSXElement<'a>) {
        let s = elem.span;
        if s.start < self.span.start || s.end > self.span.end {
            walk::walk_jsx_element(self, elem);
            return;
        }
        if let Some(resolved) = jsx_element_child(
            elem,
            self.import_table,
            self.local_components,
            self.file_path,
        ) {
            self.children.push(resolved);
        }
        walk::walk_jsx_element(self, elem);
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn jsx_element_child(
    elem: &oxc_ast::ast::JSXElement<'_>,
    import_table: &ImportTable,
    local_components: &HashMap<String, String>,
    file_path: &Path,
) -> Option<(PathBuf, String)> {
    let (root_name, member_suffix) = element_root_and_suffix(&elem.opening_element.name);
    let root = root_name?;
    resolve_target(
        &root,
        member_suffix.as_deref(),
        import_table,
        local_components,
        file_path,
    )
}

#[allow(dead_code)]
pub(crate) fn collect_jsx_children(
    program: &Program<'_>,
    import_table: &ImportTable,
    file_path: &Path,
    span: Span,
) -> Vec<(PathBuf, String)> {
    let local_components = collect_local_components(program);
    let mut visitor = JsxChildrenVisitor::new(import_table, &local_components, file_path, span);
    visitor.visit_program(program);
    visitor.children
}
