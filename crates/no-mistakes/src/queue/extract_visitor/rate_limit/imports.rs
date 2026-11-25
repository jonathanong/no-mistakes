use super::{QueueVisitor, State};
use crate::ast::js_literal;
use crate::queue::extract_helpers::{export_name, static_string};
use crate::queue::extract_job_id::referenced_symbol;
use oxc_ast::ast::{Expression, ImportDeclaration, ImportDeclarationSpecifier};
use oxc_syntax::symbol::SymbolId;

#[path = "imports_params.rs"]
mod params;

pub(super) struct Import {
    pub(super) origin: ImportOrigin,
    pub(super) value: bool,
}

pub(super) enum ImportOrigin {
    Queue(Member),
    LocalNamed { source: String, imported: String },
    LocalNamespace { source: String },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Member {
    Error,
    Worker,
    Namespace,
    Other,
}

pub(super) struct ImportedMember {
    pub(super) symbol: SymbolId,
    pub(super) member: Member,
}

pub(super) enum ThrowTarget {
    Identifier {
        symbol: Option<SymbolId>,
        is_rate_limit_name: bool,
    },
    MemberRateLimit {
        object: MemberObject,
    },
}

pub(super) enum MemberObject {
    Identifier(Option<SymbolId>),
    NamespaceWorker {
        symbol: Option<SymbolId>,
        member: String,
    },
    Opaque,
}

impl State {
    pub(super) fn matches_value(&self, member: &ImportedMember) -> bool {
        self.imports.get(&member.symbol).is_some_and(|import| {
            import.value && matches!(import.origin, ImportOrigin::Queue(m) if m == member.member)
        })
    }

    pub(super) fn matches_type(&self, member: &ImportedMember) -> bool {
        self.imports.get(&member.symbol).is_some_and(
            |import| matches!(import.origin, ImportOrigin::Queue(m) if m == member.member),
        )
    }
}

impl QueueVisitor<'_> {
    pub(in crate::queue) fn note_rate_limit_import(
        &mut self,
        decl: &ImportDeclaration<'_>,
        queue_package: bool,
    ) {
        if !self.shared_rate_limit.enabled {
            return;
        }
        let source = js_literal(&decl.source).into_owned();
        for spec in decl.specifiers.iter().flatten() {
            let (binding, origin, type_only) = match spec {
                ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                    let imported = export_name(&spec.imported);
                    let origin = if queue_package {
                        let member = match imported.as_str() {
                            "RateLimitError" => Member::Error,
                            "Worker" | "TestWorker" => Member::Worker,
                            _ => Member::Other,
                        };
                        ImportOrigin::Queue(member)
                    } else {
                        ImportOrigin::LocalNamed {
                            source: source.clone(),
                            imported,
                        }
                    };
                    (&spec.local, origin, spec.import_kind.is_type())
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                    let origin = if queue_package {
                        ImportOrigin::Queue(Member::Namespace)
                    } else {
                        ImportOrigin::LocalNamespace {
                            source: source.clone(),
                        }
                    };
                    (&spec.local, origin, false)
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                    let origin = if queue_package {
                        ImportOrigin::Queue(Member::Other)
                    } else {
                        ImportOrigin::LocalNamed {
                            source: source.clone(),
                            imported: "default".to_string(),
                        }
                    };
                    (&spec.local, origin, false)
                }
            };
            if let Some(symbol) = binding.symbol_id.get() {
                self.shared_rate_limit.imports.insert(
                    symbol,
                    Import {
                        origin,
                        value: !type_only && !decl.import_kind.is_type(),
                    },
                );
            }
        }
    }

    pub(super) fn rate_limit_error(&self, expr: &Expression<'_>) -> Option<ThrowTarget> {
        let callee = match expr.get_inner_expression() {
            Expression::NewExpression(new_expr) => &new_expr.callee,
            Expression::CallExpression(call) => &call.callee,
            _ => return None,
        };
        match callee.get_inner_expression() {
            Expression::Identifier(id) => {
                let is_rate_limit_name = id.name == "RateLimitError";
                let symbol = referenced_symbol(self, id);
                (is_rate_limit_name || symbol.is_some()).then_some(ThrowTarget::Identifier {
                    symbol,
                    is_rate_limit_name,
                })
            }
            Expression::StaticMemberExpression(member)
                if member.property.name == "RateLimitError" =>
            {
                Some(ThrowTarget::MemberRateLimit {
                    object: self.classify_member_object(&member.object),
                })
            }
            Expression::ComputedMemberExpression(member)
                if static_string(&member.expression).as_deref() == Some("RateLimitError") =>
            {
                Some(ThrowTarget::MemberRateLimit {
                    object: self.classify_member_object(&member.object),
                })
            }
            _ => None,
        }
    }

    fn classify_member_object(&self, object: &Expression<'_>) -> MemberObject {
        match object.get_inner_expression() {
            Expression::Identifier(id) => MemberObject::Identifier(referenced_symbol(self, id)),
            Expression::StaticMemberExpression(inner)
                if matches!(inner.property.name.as_str(), "Worker" | "TestWorker") =>
            {
                if let Expression::Identifier(ns) = inner.object.get_inner_expression() {
                    MemberObject::NamespaceWorker {
                        symbol: referenced_symbol(self, ns),
                        member: inner.property.name.to_string(),
                    }
                } else {
                    MemberObject::Opaque
                }
            }
            _ => MemberObject::Opaque,
        }
    }
}
