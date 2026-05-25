use crate::fetch::types::{CacheKind, FetchOccurrence};
use oxc_span::Span;
use std::collections::HashSet;

pub struct FetchVisitor<'a> {
    pub source: &'a str,
    pub file: String,
    pub fetches: Vec<FetchOccurrence>,
    pub is_client: bool,
    pub is_route_handler: bool,
    pub cached_function: Option<String>,
    pub cached_kind: Option<CacheKind>,
    pub fetch_scope_stack: Vec<FetchScope>,
    pub in_var_declaration: bool,
    pub component_span: Option<Span>,
    pub function_name_stack: Vec<Option<String>>,
    pub conditional_depth: u32,
    pub promise_all_depth: u32,
    pub try_depth: u32,
    pub pending_var_name: Option<String>,
}

#[derive(Default)]
pub struct FetchScope {
    pub shadowed_identifiers: HashSet<String>,
    pub tracks_var_bindings: bool,
}

impl<'a> FetchVisitor<'a> {
    pub fn new(source: &'a str, file: &str, is_client: bool, is_route_handler: bool) -> Self {
        Self {
            source,
            file: file.to_string(),
            fetches: Vec::new(),
            is_client,
            is_route_handler,
            cached_function: None,
            cached_kind: None,
            fetch_scope_stack: vec![FetchScope {
                shadowed_identifiers: HashSet::new(),
                tracks_var_bindings: true,
            }],
            in_var_declaration: false,
            component_span: None,
            function_name_stack: Vec::new(),
            conditional_depth: 0,
            promise_all_depth: 0,
            try_depth: 0,
            pending_var_name: None,
        }
    }

    pub fn current_function_name(&self) -> Option<String> {
        self.function_name_stack
            .iter()
            .rev()
            .find_map(|n| n.clone())
    }

    pub fn enter_fetch_scope(&mut self, tracks_var_bindings: bool) {
        self.fetch_scope_stack.push(FetchScope {
            shadowed_identifiers: HashSet::new(),
            tracks_var_bindings,
        });
    }

    pub fn leave_fetch_scope(&mut self) {
        self.fetch_scope_stack.pop();
    }

    pub fn mark_fetch_shadowed(&mut self) {
        if let Some(scope) = self.fetch_scope_stack.last_mut() {
            scope.shadowed_identifiers.insert("fetch".to_string());
        }
    }

    #[inline(never)]
    pub fn mark_identifier_shadowed_in_var_scope(&mut self, name: &str) {
        for scope in self.fetch_scope_stack.iter_mut().rev() {
            if scope.tracks_var_bindings {
                scope.shadowed_identifiers.insert(name.to_string());
                return;
            }
        }

        if let Some(scope) = self.fetch_scope_stack.last_mut() {
            scope.shadowed_identifiers.insert(name.to_string());
        }
    }

    pub fn mark_identifier_shadowed(&mut self, name: &str) {
        if let Some(scope) = self.fetch_scope_stack.last_mut() {
            scope.shadowed_identifiers.insert(name.to_string());
        }
    }

    pub fn is_fetch_shadowed(&self) -> bool {
        self.fetch_scope_stack
            .iter()
            .any(|scope| scope.shadowed_identifiers.contains("fetch"))
    }
}
