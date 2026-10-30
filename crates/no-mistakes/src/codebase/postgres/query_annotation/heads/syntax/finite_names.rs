//! Bind closed const names after the walk, and record import-gated templates.
use super::{span, Syntax};
use crate::codebase::postgres::query_annotation::heads::finite_comments::{
    self, Domain, PendingName, TemplateUse,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingIdentifier, CallExpression, Expression};

impl Syntax {
    pub(super) fn record_finite_name(&mut self, call: &CallExpression<'_>) {
        let Some(expression) = call.arguments.first().and_then(|arg| arg.as_expression()) else {
            return;
        };
        if let Some((reference, before, after)) = finite_comments::template(expression) {
            self.record_identifier_name(call, expression, reference, before, after);
            return;
        }
        let Some(inline) = finite_comments::inline_name(expression) else {
            return;
        };
        self.finite_templates.push(TemplateUse {
            call: span(call.span),
            reference: (0, 0),
            domain: Domain {
                declaration: (0, 0),
                name: String::new(),
                values: inline.values,
                import_gated: true,
            },
            before: inline.before,
            after: inline.after,
            inline: true,
        });
    }

    pub(super) fn record_closed_name(
        &mut self,
        binding: &BindingIdentifier<'_>,
        init: &Expression<'_>,
    ) {
        let Some(values) = finite_comments::plain_ternary(init) else {
            return;
        };
        self.name_domains.push(Domain {
            declaration: span(binding.span),
            name: binding.name.to_string(),
            values,
            import_gated: true,
        });
    }

    pub(super) fn bind_closed_names(&mut self) {
        if self.pending_name_templates.is_empty() || self.name_domains.is_empty() {
            return;
        }
        let pending = std::mem::take(&mut self.pending_name_templates);
        let mut matched = Vec::new();
        for item in &pending {
            for domain in self
                .name_domains
                .iter()
                .filter(|domain| domain.name == item.name)
            {
                matched.push(TemplateUse {
                    call: item.call,
                    reference: item.reference,
                    domain: domain.clone(),
                    before: item.before.clone(),
                    after: item.after.clone(),
                    inline: false,
                });
            }
        }
        self.finite_templates.extend(matched);
    }

    fn record_identifier_name(
        &mut self,
        call: &CallExpression<'_>,
        expression: &Expression<'_>,
        reference: super::super::Span,
        before: String,
        after: String,
    ) {
        let name = identifier_name(expression);
        let matched = self
            .active_domains
            .iter()
            .rev()
            .find(|domain| name == Some(domain.name.as_str()))
            .cloned();
        if let Some(domain) = matched {
            self.finite_templates.push(TemplateUse {
                call: span(call.span),
                reference,
                domain,
                before,
                after,
                inline: false,
            });
            return;
        }
        let Some(name) = name else {
            return;
        };
        self.pending_name_templates.push(PendingName {
            call: span(call.span),
            reference,
            name: name.to_string(),
            before,
            after,
        });
    }
}

fn identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    let Expression::TemplateLiteral(value) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    let Expression::Identifier(identifier) = value.expressions.first()? else {
        return None;
    };
    Some(identifier.name.as_str())
}
