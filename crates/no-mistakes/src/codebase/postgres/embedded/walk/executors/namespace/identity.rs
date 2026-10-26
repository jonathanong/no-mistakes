use super::super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(in crate::codebase::postgres::embedded::walk) enum Origin {
    Binding(u32, String),
    Module(String),
    Export(String, String),
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn receiver_origins(
        &self,
        expression: &Expression<'_>,
    ) -> Vec<Origin> {
        match unwrap_ts_wrappers(expression) {
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                if let Some((index, scope)) = self
                    .scopes
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, scope)| scope.contains_key(name))
                {
                    let key = (self.member_query_proof.scope_ids[index], name.to_string());
                    if let Some(origins) = self.member_query_proof.aliases.get(&key) {
                        return origins.clone();
                    }
                    return if scope[name].executor_alias.is_some()
                        || self.relative_proofs[index].contains_key(name)
                        || self.scoped.delayed.contains(&identifier.span.start)
                    {
                        vec![Origin::Binding(key.0, key.1)]
                    } else {
                        Vec::new()
                    };
                }
                if let Some(module) = self.imported_namespace_names.get(name) {
                    return vec![Origin::Module(module.clone())];
                }
                if let Some((module, export)) = self.imported_value_origins.get(name) {
                    return vec![Origin::Export(module.clone(), export.clone())];
                }
                if self.bindings.contains(name) {
                    vec![Origin::Binding(0, name.to_string())]
                } else {
                    Vec::new()
                }
            }
            Expression::CallExpression(call) if self.configured_require(call) => {
                let module = call
                    .arguments
                    .first()
                    .and_then(|arg| arg.as_expression())
                    .and_then(|arg| match arg {
                        Expression::StringLiteral(value) => {
                            Some(crate::ast::js_literal(&value).into_owned())
                        }
                        _ => None,
                    });
                module
                    .map(|name| vec![Origin::Module(name)])
                    .unwrap_or_default()
            }
            Expression::StaticMemberExpression(member) => self
                .receiver_origins(&member.object)
                .into_iter()
                .filter_map(|origin| match origin {
                    Origin::Module(module) => {
                        Some(Origin::Export(module, member.property.name.to_string()))
                    }
                    _ => None,
                })
                .collect(),
            Expression::ComputedMemberExpression(member) => {
                let Expression::StringLiteral(property) = unwrap_ts_wrappers(&member.expression)
                else {
                    return Vec::new();
                };
                self.receiver_origins(&member.object)
                    .into_iter()
                    .filter_map(|origin| match origin {
                        Origin::Module(module) => Some(Origin::Export(
                            module,
                            crate::ast::js_literal(&property).into_owned(),
                        )),
                        _ => None,
                    })
                    .collect()
            }
            Expression::ConditionalExpression(value) => {
                let mut origins = self.receiver_origins(&value.consequent);
                for origin in self.receiver_origins(&value.alternate) {
                    if !origins.contains(&origin) {
                        origins.push(origin);
                    }
                }
                origins
            }
            _ => Vec::new(),
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn bind_receiver_alias(
        &mut self,
        name: &str,
        expression: &Expression<'_>,
    ) {
        let origins = self.receiver_origins(expression);
        self.set_receiver_alias_origins(name, origins);
    }

    pub(in crate::codebase::postgres::embedded::walk) fn receiver_alias_origins(
        &self,
        name: &str,
    ) -> Vec<Origin> {
        self.scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))
            .and_then(|(index, _)| {
                self.member_query_proof
                    .aliases
                    .get(&(self.member_query_proof.scope_ids[index], name.to_string()))
            })
            .cloned()
            .unwrap_or_default()
    }

    pub(in crate::codebase::postgres::embedded::walk) fn set_receiver_alias_origins(
        &mut self,
        name: &str,
        origins: Vec<Origin>,
    ) {
        let Some((index, _)) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))
        else {
            return;
        };
        let key = (self.member_query_proof.scope_ids[index], name.to_string());
        if origins.is_empty() {
            self.member_query_proof.aliases.remove(&key);
        } else {
            self.member_query_proof.aliases.insert(key, origins);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn forget_receiver_alias(
        &mut self,
        name: &str,
    ) {
        if let Some((index, _)) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))
        {
            let key = (self.member_query_proof.scope_ids[index], name.to_string());
            self.member_query_proof.aliases.remove(&key);
        }
    }
}
