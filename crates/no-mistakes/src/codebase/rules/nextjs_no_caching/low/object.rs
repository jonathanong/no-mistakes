use super::{literal, Collector, Context};
use oxc_ast::ast::*;

impl<'a> Collector<'a> {
    pub(super) fn object(&mut self, object: &ObjectExpression<'a>, context: Context) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                self.unknown(object.span.start);
                continue;
            };
            let Some(name) = super::super::const_values::property_name(&property.key) else {
                self.unknown(property.span.start);
                continue;
            };
            match (context, name.as_str()) {
                (Context::Fetch, "next") | (Context::Config, "experimental") => {
                    if let Expression::ObjectExpression(nested) = &property.value {
                        self.object(
                            nested,
                            if matches!(context, Context::Fetch) {
                                Context::Next
                            } else {
                                Context::Config
                            },
                        );
                    } else {
                        self.unknown(property.span.start);
                    }
                }
                (Context::Fetch, "cache")
                | (Context::Next, "revalidate")
                | (Context::Config, "cacheComponents") => {
                    if !literal(&property.value) {
                        self.unknown(property.span.start);
                    }
                }
                (Context::Next, "tags")
                    if !matches!(property.value, Expression::ArrayExpression(_)) =>
                {
                    self.unknown(property.span.start);
                }
                _ => {}
            }
        }
    }
}
