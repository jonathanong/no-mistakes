use super::ScopeCollector;
use oxc_ast::ast::{BindingPattern, TSType, VariableDeclarator};
use oxc_span::GetSpan;

use super::{ImportClassification, ScopedExecutors};

impl ScopeCollector {
    pub(in crate::codebase::postgres::embedded::scoped_bindings) fn from_imports(
        imports: ImportClassification,
    ) -> Self {
        Self {
            factories: imports.factories,
            types: imports.types,
            found: ScopedExecutors {
                undeclared_imports: imports.undeclared_runtime,
                ..Default::default()
            },
            undeclared: imports.undeclared,
            provisional_factories: imports.provisional_factories,
            provisional_types: imports.provisional_types,
            ..Self::default()
        }
    }

    pub(super) fn record_undeclared(&mut self, name: &str, start: u32, ty: &TSType<'_>) {
        self.record_path_annotation(name, start, ty, "", None);
    }

    pub(super) fn record_undeclared_variable(&mut self, declaration: &VariableDeclarator<'_>) {
        if let BindingPattern::BindingIdentifier(ident) = &declaration.id {
            if let Some(annotation) = &declaration.type_annotation {
                self.record_undeclared(
                    ident.name.as_str(),
                    declaration.id.span().start,
                    &annotation.type_annotation,
                );
            }
        }
    }
}
