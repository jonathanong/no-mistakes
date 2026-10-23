//! Reuse the SQL walk's binding frames for diagnostic-only imported identities.
use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(super) fn bind_undeclared_declaration(&mut self, name: &str, start: u32) {
        if let Some(identity) = self
            .scoped
            .undeclared
            .get(&start)
            .and_then(|names| names.get(name))
        {
            let index = self
                .scopes
                .iter()
                .rposition(|scope| scope.contains_key(name))
                .expect("binding scope");
            self.undeclared_scopes[index].insert(name.to_string(), identity.clone());
        }
    }

    fn undeclared_identity(
        &self,
        name: &str,
        path: &str,
    ) -> Option<&super::super::UndeclaredExecutorType> {
        match self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
        {
            Some(index) => self.undeclared_scopes[index]
                .get(name)
                .and_then(|types| types.get(path)),
            None if path.is_empty() => self.scoped.undeclared_imports.get(name),
            None => None,
        }
    }

    pub(super) fn bind_undeclared_alias(&mut self, name: &str, init: &Expression<'_>) {
        let Some(index) = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
        else {
            // Some helper-only declarators have no ordinary value binding.
            return;
        };
        if self.undeclared_scopes[index].contains_key(name) {
            return;
        }
        let source_name = match unwrap_ts_wrappers(init) {
            Expression::Identifier(alias) => Some(alias.name.as_str()),
            Expression::NewExpression(new) => match unwrap_ts_wrappers(&new.callee) {
                Expression::Identifier(constructor) => Some(constructor.name.as_str()),
                _ => None,
            },
            _ => None,
        };
        if let Some(source_name) = source_name {
            let mut types = self
                .scopes
                .iter()
                .rposition(|scope| scope.contains_key(source_name))
                .and_then(|index| self.undeclared_scopes[index].get(source_name))
                .cloned()
                .unwrap_or_default();
            if let Some(identity) = self.undeclared_identity(source_name, "") {
                types.insert(String::new(), identity.clone());
            }
            if types.is_empty() {
                return;
            }
            for identity in types.values_mut() {
                identity.explicit_annotation = false;
            }
            self.undeclared_scopes[index].insert(name.to_string(), types);
        }
    }

    pub(super) fn clear_undeclared_binding(&mut self, name: &str) {
        if let Some(index) = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
        {
            // Assignment changes an inferred value identity, not a declared type.
            if self.undeclared_scopes[index]
                .get(name)
                .is_some_and(|types| types.values().all(|identity| !identity.explicit_annotation))
            {
                self.undeclared_scopes[index].remove(name);
            }
        }
    }

    pub(super) fn record_undeclared_executor_call(&mut self, call: &CallExpression<'_>) {
        let (receiver, method) = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) => {
                (&member.object, member.property.name.as_str())
            }
            Expression::ComputedMemberExpression(member) => {
                let Some(key) = static_method_name(&member.expression) else {
                    return;
                };
                (&member.object, key)
            }
            _ => return,
        };
        if !self.executor_methods.iter().any(|name| name == method) {
            return;
        }
        let Some((name, path)) = super::undeclared_receivers::receiver_path(receiver) else {
            return;
        };
        let identity = if name == "this" {
            let Some((scope, types)) = self
                .scoped
                .undeclared_this
                .iter()
                .filter(|(scope, _)| scope.start <= call.span.start && call.span.end <= scope.end)
                .min_by_key(|(scope, _)| scope.end - scope.start)
            else {
                return;
            };
            if self.scoped.undeclared_this_exclusions.iter().any(|span| {
                scope.start <= span.start
                    && span.start <= call.span.start
                    && call.span.end <= span.end
            }) {
                return;
            }
            types.get(&path)
        } else {
            self.undeclared_identity(name, &path)
        };
        if let Some(identity) = identity {
            // Configured imports and declared receivers already have executor proof.
            if self.proven_executor(receiver)
                && (!self.namespace_expression(receiver) || self.query_binding)
            {
                return;
            }
            let mut occurrence = identity.clone();
            occurrence.line = crate::codebase::ts_source::byte_offset_to_line(
                self.source,
                call.span.start as usize,
            );
            occurrence.span = (call.span.start, call.span.end);
            self.undeclared_executor_types.push(occurrence);
        }
    }
}

fn static_method_name<'e>(expression: &'e Expression<'_>) -> Option<&'e str> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(key) => Some(key.value.as_str()),
        Expression::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            let value = &template.quasis[0].value;
            Some(
                value
                    .cooked
                    .as_ref()
                    .map(|cooked| cooked.as_str())
                    .unwrap_or(value.raw.as_str()),
            )
        }
        _ => None,
    }
}

pub(super) fn methods(options: &super::super::EmbeddedSqlOptions) -> Vec<String> {
    let mut methods = if options.executor_names.is_empty() {
        vec!["query".into(), "read".into(), "write".into()]
    } else {
        options.executor_names.clone()
    };
    // Scoped .query receiver matching is independent of direct import names.
    if !options.import_specifier.is_empty() && !methods.iter().any(|name| name == "query") {
        methods.push("query".into());
    }
    methods
}
