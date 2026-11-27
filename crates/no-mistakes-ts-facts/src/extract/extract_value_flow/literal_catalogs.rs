use super::*;
use oxc_ast::ast::{BindingPattern, Expression, IdentifierReference, ReturnStatement};

#[path = "literal_catalog_callbacks.rs"]
mod callbacks;
#[path = "literal_catalog_reads.rs"]
mod reads;
#[path = "literal_catalog_shapes.rs"]
mod shapes;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LiteralCatalogFacts {
    pub sources: Vec<LiteralCatalog>,
    pub accessors: Vec<CatalogAccessor>,
    pub callbacks: FxHashMap<CallableId, CatalogCallback>,
    pub reads: FxHashMap<u32, CatalogRead>,
    pub consumers: FxHashMap<u32, (u32, u32)>,
    pub literal_receivers: FxHashSet<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteralCatalog {
    pub name: String,
    pub scope: usize,
    pub declared_at: u32,
    pub record: bool,
    pub paths: Vec<Vec<String>>,
    pub fields: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogAccessor {
    pub owner: CallableId,
    pub source: String,
    pub scope: usize,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCallback {
    pub reads: Vec<Vec<String>>,
    pub string_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRead {
    pub name: String,
    pub scope: usize,
    pub path: Vec<String>,
    pub kind: CatalogReadKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogReadKind {
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
    pub fn note_literal_catalog_binding(
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

    pub fn note_literal_catalog_return(&mut self, statement: &ReturnStatement<'_>) {
        if !self.collect_value_flow {
            return;
        }
        let (Some(owner), Some(argument)) = (self.current_function_id(), &statement.argument)
        else {
            return;
        };
        let member = match crate::syntax_helpers::unwrap_ts_wrappers(argument) {
            Expression::ComputedMemberExpression(member) if !member.optional => member,
            _ => return,
        };
        let (Expression::Identifier(source), Expression::Identifier(key)) = (
            crate::syntax_helpers::unwrap_ts_wrappers(&member.object),
            crate::syntax_helpers::unwrap_ts_wrappers(&member.expression),
        ) else {
            return;
        };
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

    pub fn note_literal_catalog_identifier(&mut self, identifier: &IdentifierReference<'_>) {
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

#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
#[path = "literal_catalogs_tests.rs"]
mod tests;
