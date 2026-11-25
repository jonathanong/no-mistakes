use super::{QueueVisitor, State};
use crate::queue::extract_helpers::export_name;
use crate::queue::extract_job_id::referenced_symbol;
use oxc_ast::ast::{
    Argument, BindingPattern, Expression, FormalParameters, ImportDeclaration,
    ImportDeclarationSpecifier, TSType, TSTypeName,
};
use oxc_syntax::symbol::SymbolId;

pub(super) struct Import {
    member: Member,
    value: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Member {
    Error,
    Worker,
    Namespace,
    WorkerOrNamespace,
}

pub(super) struct ImportedMember {
    symbol: SymbolId,
    member: Member,
}

impl State {
    pub(super) fn matches_value(&self, member: &ImportedMember) -> bool {
        self.imports.get(&member.symbol).is_some_and(|import| {
            let matches = match member.member {
                Member::WorkerOrNamespace => {
                    matches!(import.member, Member::Worker | Member::Namespace)
                }
                expected => import.member == expected,
            };
            import.value && matches
        })
    }

    pub(super) fn matches_type(&self, member: &ImportedMember) -> bool {
        self.imports
            .get(&member.symbol)
            .is_some_and(|import| import.member == member.member)
    }
}

impl QueueVisitor<'_> {
    pub(in crate::queue) fn note_rate_limit_import(
        &mut self,
        decl: &ImportDeclaration<'_>,
        queue_package: bool,
    ) {
        if !queue_package || !self.shared_rate_limit.enabled {
            return;
        }
        for spec in decl.specifiers.iter().flatten() {
            let (binding, member, type_only) = match spec {
                ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    let member = match export_name(&spec.imported).as_str() {
                        "RateLimitError" => Member::Error,
                        "Worker" | "TestWorker" => Member::Worker,
                        _ => continue,
                    };
                    (&spec.local, member, spec.import_kind.is_type())
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    (&spec.local, Member::Namespace, false)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => continue,
            };
            if let Some(symbol) = binding.symbol_id.get() {
                self.shared_rate_limit.imports.insert(
                    symbol,
                    Import {
                        member,
                        value: !type_only && !decl.import_kind.is_type(),
                    },
                );
            }
        }
    }

    pub(super) fn rate_limit_error(&self, expr: &Expression<'_>) -> Option<ImportedMember> {
        let callee = match expr.get_inner_expression() {
            Expression::NewExpression(new_expr) => &new_expr.callee,
            Expression::CallExpression(call) => &call.callee,
            _ => return None,
        };
        match callee.get_inner_expression() {
            Expression::Identifier(_) => self.rate_limit_imported_member(callee, Member::Error),
            Expression::StaticMemberExpression(member)
                if member.property.name == "RateLimitError" =>
            {
                if matches!(
                    member.object.get_inner_expression(),
                    Expression::Identifier(_)
                ) {
                    self.rate_limit_imported_member(&member.object, Member::WorkerOrNamespace)
                } else {
                    self.worker_member(&member.object)
                }
            }
            _ => None,
        }
    }

    pub(in crate::queue) fn note_constructor_rate_limit_processor(
        &mut self,
        callee: &Expression<'_>,
        argument: &Argument<'_>,
        worker: usize,
    ) {
        if self.shared_rate_limit.enabled
            && self
                .worker_member(callee)
                .is_some_and(|import| self.shared_rate_limit.matches_value(&import))
        {
            self.note_rate_limit_processor(argument, worker, false);
        }
    }

    fn worker_member(&self, expr: &Expression<'_>) -> Option<ImportedMember> {
        if let Expression::StaticMemberExpression(member) = expr.get_inner_expression() {
            if matches!(member.property.name.as_str(), "Worker" | "TestWorker") {
                return self.rate_limit_imported_member(&member.object, Member::Namespace);
            }
        }
        self.rate_limit_imported_member(expr, Member::Worker)
    }

    fn rate_limit_imported_member(
        &self,
        expr: &Expression<'_>,
        member: Member,
    ) -> Option<ImportedMember> {
        let Expression::Identifier(id) = expr.get_inner_expression() else {
            return None;
        };
        Some(ImportedMember {
            symbol: referenced_symbol(self, id)?,
            member,
        })
    }

    pub(super) fn note_worker_parameters(&mut self, start: u32, params: &FormalParameters<'_>) {
        for param in &params.items {
            let Some(annotation) = &param.type_annotation else {
                continue;
            };
            let TSType::TSTypeReference(reference) = &annotation.type_annotation else {
                continue;
            };
            let TSTypeName::IdentifierReference(name) = &reference.type_name else {
                continue;
            };
            let BindingPattern::BindingIdentifier(binding) = &param.pattern else {
                continue;
            };
            if let (Some(parameter), Some(symbol)) = (
                self.stable_rate_limit_symbol(binding),
                referenced_symbol(self, name),
            ) {
                self.shared_rate_limit.parameters.insert(
                    parameter,
                    (
                        start,
                        ImportedMember {
                            symbol,
                            member: Member::Worker,
                        },
                    ),
                );
            }
        }
    }
}
