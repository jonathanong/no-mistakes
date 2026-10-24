use super::ancestry::{NamespaceBody, NamespaceValue};
use crate::codebase::dependencies::extract::{statement_value_names, CallableId};
use oxc_ast::ast::*;

pub(super) fn collect(
    block: &TSModuleBlock<'_>,
    namespace: &[String],
    ambient: bool,
) -> NamespaceBody {
    // TS binder's ExportContext: ordinary export modifiers preserve implicit
    // ambient exports; export clauses and assignments make them explicit.
    let implicit = ambient
        && !block.body.iter().any(|statement| {
            matches!(
                statement,
                Statement::ExportNamedDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
                    | Statement::TSExportAssignment(_)
                    | Statement::ExportDefaultDeclaration(_)
            )
        });
    let values = block
        .body
        .iter()
        .flat_map(|statement| {
            let class = match statement {
                Statement::ClassDeclaration(class) => Some(CallableId(class.span.start)),
                Statement::ExportDeclaration(export) => match &export.declaration {
                    Declaration::ClassDeclaration(class) => Some(CallableId(class.span.start)),
                    _ => None,
                },
                _ => None,
            };
            let exported = matches!(statement, Statement::ExportDeclaration(_))
                || implicit && !matches!(statement, Statement::TSImportEqualsDeclaration(_));
            statement_value_names(statement)
                .into_iter()
                .map(move |name| NamespaceValue {
                    name,
                    class,
                    exported,
                })
        })
        .collect();
    NamespaceBody {
        id: block.span.start,
        namespace: namespace.to_vec(),
        values,
    }
}
