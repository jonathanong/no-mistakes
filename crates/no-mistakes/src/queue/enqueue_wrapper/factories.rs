use crate::config::v2::schema::{EnqueueFactory, QueueFactoryRoles};
use crate::fx::FxHashMap;
use crate::queue::extract_helpers::export_name;
use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;

pub(in crate::queue) type Factories<'a> = FxHashMap<SymbolId, &'a EnqueueFactory>;

pub(crate) fn needs_scoping(source: &str, roles: &QueueFactoryRoles) -> bool {
    roles.enqueue.iter().any(|role| {
        role.positional().is_none() && (source.contains(&role.callee) || source.contains('\\'))
    })
}

pub(in crate::queue) fn factories<'a>(
    program: &Program<'_>,
    roles: &'a QueueFactoryRoles,
    scoping: Option<&Scoping>,
) -> Factories<'a> {
    let mut factories = Factories::default();
    if scoping.is_none() || roles.enqueue.iter().all(|role| role.positional().is_some()) {
        return factories;
    }
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        for spec in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(spec) = spec else {
                continue;
            };
            if spec.import_kind.is_type() {
                continue;
            }
            let imported = export_name(&spec.imported);
            if let (Some(symbol), Some(role)) = (
                spec.local.symbol_id.get(),
                roles
                    .enqueue
                    .iter()
                    .find(|role| role.callee == imported && role.positional().is_none()),
            ) {
                factories.insert(symbol, role);
            }
        }
    }
    factories
}

#[path = "exports.rs"]
mod exports;
pub(in crate::queue) use exports::collect_exports;
