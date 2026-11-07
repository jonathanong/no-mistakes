use super::*;
use oxc_ast::ast::{BindingPattern, Expression, IdentifierReference, ReturnStatement};

#[path = "literal_catalog_callbacks.rs"]
mod callbacks;
#[path = "literal_catalog_reads.rs"]
mod reads;
#[path = "literal_catalog_shapes.rs"]
mod shapes;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LiteralCatalogFacts {
    pub(crate) sources: Vec<LiteralCatalog>,
    pub(crate) accessors: Vec<CatalogAccessor>,
    pub(crate) callbacks: FxHashMap<CallableId, CatalogCallback>,
    pub(crate) reads: FxHashMap<u32, CatalogRead>,
    pub(crate) consumers: FxHashMap<u32, (u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiteralCatalog {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) declared_at: u32,
    pub(crate) record: bool,
    pub(crate) paths: Vec<Vec<String>>,
    pub(crate) fields: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogAccessor {
    pub(crate) owner: CallableId,
    pub(crate) source: String,
    pub(crate) scope: usize,
    pub(crate) key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogCallback {
    pub(crate) reads: Vec<Vec<String>>,
    pub(crate) string_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogRead {
    pub(crate) name: String,
    pub(crate) scope: usize,
    pub(crate) path: Vec<String>,
    pub(crate) kind: CatalogReadKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CatalogReadKind {
    Value,
    Collection(u32, u32),
    Callee,
    Lookup(CallableId),
    Invalid,
}

impl ImportCollector {
    fn catalog_binding_scope(&self, name: &str) -> Option<usize> {
        self.callee_binding_scope(name).or_else(|| {
            (self.imported_bindings.contains(name)
                || self.predeclared_imported_bindings.contains(name))
            .then_some(0)
        })
    }
    pub(crate) fn note_literal_catalog_binding(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
        declared_at: u32,
    ) {
        if !self.collect_value_flow || self.namespace.type_depth > 0 {
            return;
        }
        let BindingPattern::BindingIdentifier(identifier) = pattern else {
            return;
        };
        let Some((record, paths, fields)) = shapes::catalog(init) else {
            return;
        };
        self.value_flow.catalogs.sources.push(LiteralCatalog {
            name: identifier.name.to_string(),
            scope: self
                .callee_binding_scope(identifier.name.as_str())
                .unwrap_or(0),
            declared_at,
            record,
            paths,
            fields,
        });
    }

    pub(crate) fn note_literal_catalog_return(&mut self, statement: &ReturnStatement<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let (Some(owner), Some(argument)) = (self.current_function_id(), &statement.argument)
        else {
            return;
        };
        let Expression::ComputedMemberExpression(member) =
            crate::codebase::ts_source::unwrap_ts_wrappers(argument)
        else {
            return;
        };
        let (Expression::Identifier(source), Expression::Identifier(key)) = (
            crate::codebase::ts_source::unwrap_ts_wrappers(&member.object),
            crate::codebase::ts_source::unwrap_ts_wrappers(&member.expression),
        ) else {
            return;
        };
        if member.optional {
            return;
        }
        let Some(scope) = self.callee_binding_scope(source.name.as_str()) else {
            return;
        };
        self.value_flow.catalogs.accessors.push(CatalogAccessor {
            owner,
            source: source.name.to_string(),
            scope,
            key: key.name.to_string(),
        });
        self.value_flow.catalogs.reads.insert(
            source.span.start,
            CatalogRead {
                name: source.name.to_string(),
                scope,
                path: Vec::new(),
                kind: CatalogReadKind::Lookup(owner),
            },
        );
    }

    pub(crate) fn note_literal_catalog_identifier(&mut self, identifier: &IdentifierReference<'_>) {
        if !self.collect_value_flow || self.namespace.type_depth > 0 {
            return;
        }
        let Some(scope) = self.catalog_binding_scope(identifier.name.as_str()) else {
            return;
        };
        self.value_flow
            .catalogs
            .reads
            .entry(identifier.span.start)
            .or_insert_with(|| CatalogRead {
                name: identifier.name.to_string(),
                scope,
                path: Vec::new(),
                kind: CatalogReadKind::Invalid,
            });
    }
}
