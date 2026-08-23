use crate::react_traits::analyze::components::ComponentDef;
use crate::react_traits::analyze::import_table::ImportTable;
use crate::react_traits::analyze::jsx_children::jsx_element_child;
use crate::react_traits::analyze::jsx_resolve::collect_local_components;
use crate::react_traits::traits::{context, memo, props, state, suspense};
use oxc_ast::ast::Program;
use oxc_ast_visit::{walk, Visit};
use oxc_span::Span;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(crate) struct FileTraitHits {
    pub has_state: Vec<bool>,
    pub passes_props: Vec<bool>,
    pub uses_memo: Vec<bool>,
    pub uses_context_provider: Vec<bool>,
    pub uses_suspense_jsx: Vec<bool>,
    pub has_props: Vec<bool>,
    pub children: Vec<Vec<(PathBuf, String)>>,
}

fn within(node_span: Span, component_span: Span) -> bool {
    node_span.start >= component_span.start && node_span.end <= component_span.end
}

struct FileTraitVisitor<'a> {
    spans: &'a [Span],
    hits: FileTraitHits,
    import_table: &'a ImportTable,
    local_components: &'a HashMap<String, String>,
    file_path: &'a Path,
    dynamic_names: &'a [HashSet<String>],
}

impl FileTraitVisitor<'_> {
    fn containing_indices(&self, node: Span) -> Vec<usize> {
        self.spans
            .iter()
            .enumerate()
            .filter_map(|(i, span)| within(node, *span).then_some(i))
            .collect()
    }
}

impl<'a> Visit<'a> for FileTraitVisitor<'a> {
    fn visit_call_expression(&mut self, expr: &oxc_ast::ast::CallExpression<'a>) {
        let sets_state = state::call_sets_state(expr);
        let uses_memo = memo::call_is_use_memo(expr);
        if sets_state || uses_memo {
            for i in self.containing_indices(expr.span) {
                self.hits.has_state[i] |= sets_state;
                self.hits.uses_memo[i] |= uses_memo;
            }
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if state::member_is_this_state(expr) {
            for i in self.containing_indices(expr.span) {
                self.hits.has_state[i] = true;
            }
        }
        walk::walk_static_member_expression(self, expr);
    }

    fn visit_jsx_opening_element(&mut self, elem: &oxc_ast::ast::JSXOpeningElement<'a>) {
        let passes_props = props::jsx_passes_component_props(elem);
        let context = context::jsx_is_context_provider(elem);
        if passes_props || context {
            for i in self.containing_indices(elem.span) {
                self.hits.passes_props[i] |= passes_props;
                self.hits.uses_context_provider[i] |= context;
            }
        }
        for i in self.containing_indices(elem.span) {
            if suspense::jsx_opening_is_suspense(elem, &self.dynamic_names[i]) {
                self.hits.uses_suspense_jsx[i] = true;
            }
        }
        walk::walk_jsx_opening_element(self, elem);
    }

    fn visit_jsx_element(&mut self, elem: &oxc_ast::ast::JSXElement<'a>) {
        if let Some(resolved) = jsx_element_child(
            elem,
            self.import_table,
            self.local_components,
            self.file_path,
        ) {
            for i in self.containing_indices(elem.span) {
                self.hits.children[i].push(resolved.clone());
            }
        }
        walk::walk_jsx_element(self, elem);
    }
}

pub(crate) fn collect_file_trait_hits(
    program: &Program<'_>,
    defs: &[ComponentDef],
    dynamic_names: &[HashSet<String>],
    import_table: &ImportTable,
    file_path: &Path,
) -> FileTraitHits {
    crate::diagnostics::record_ast_walk();
    let spans: Vec<Span> = defs.iter().map(|def| def.span).collect();
    let n = spans.len();
    let local_components = collect_local_components(program);
    let mut visitor = FileTraitVisitor {
        spans: &spans,
        hits: FileTraitHits {
            has_state: vec![false; n],
            passes_props: vec![false; n],
            uses_memo: vec![false; n],
            uses_context_provider: vec![false; n],
            uses_suspense_jsx: vec![false; n],
            has_props: vec![false; n],
            children: vec![Vec::new(); n],
        },
        import_table,
        local_components: &local_components,
        file_path,
        dynamic_names,
    };
    visitor.visit_program(program);
    let mut hits = visitor.hits;
    declaration::fill_declaration_traits(program, defs, &mut hits);
    hits
}

mod declaration;

#[cfg(test)]
mod tests;
