use super::bindings::Binding;
use super::bindings_calls::inline_require_file_url_to_path;
use super::paths::{static_new_module_url, static_string};
use super::{ResourcePath, ResourcePathBase, ResourceVisitor};
use oxc_ast::ast::{Argument, CallExpression, Expression};

impl<'a> ResourceVisitor<'a> {
    pub(super) fn static_resource_path(&self, arg: &Argument<'_>) -> Option<ResourcePath> {
        static_string(arg)
            .map(|value| ResourcePath {
                value,
                base: ResourcePathBase::AnalysisRoot,
            })
            .or_else(|| self.static_module_url(arg))
            .or_else(|| self.static_file_url_to_path(arg))
    }

    fn static_module_url(&self, arg: &Argument<'_>) -> Option<ResourcePath> {
        let Argument::NewExpression(new) = arg else {
            return None;
        };
        self.static_new_module_url(new)
    }

    fn static_new_module_url(&self, new: &oxc_ast::ast::NewExpression<'_>) -> Option<ResourcePath> {
        let Expression::Identifier(callee) = &new.callee else {
            return None;
        };
        let name = callee.name.as_str();
        (!self.is_shadowed(name)
            && (name == "URL" || matches!(self.binding(name), Some(Binding::UrlConstructor))))
        .then(|| static_new_module_url(new))
        .flatten()
    }

    fn static_file_url_to_path(&self, arg: &Argument<'_>) -> Option<ResourcePath> {
        let Argument::CallExpression(call) = arg else {
            return None;
        };
        self.static_file_url_call(call)
    }

    pub(super) fn static_glob_cwd(
        &self,
        argument: Option<&Argument<'_>>,
    ) -> Result<Option<ResourcePath>, ()> {
        let Some(Argument::ObjectExpression(object)) = argument else {
            return if argument.is_none() {
                Ok(None)
            } else {
                Err(())
            };
        };
        let mut cwd = None;
        for property in &object.properties {
            let oxc_ast::ast::ObjectPropertyKind::ObjectProperty(property) = property else {
                return Err(());
            };
            let Some(key) = property.key.static_name() else {
                return Err(());
            };
            if key.as_ref() == "cwd" {
                cwd = Some(self.static_resource_expression(&property.value).ok_or(()));
            }
        }
        cwd.transpose()
    }

    fn static_resource_expression(&self, expression: &Expression<'_>) -> Option<ResourcePath> {
        match expression {
            Expression::StringLiteral(value) => Some(ResourcePath {
                value: value.value.to_string(),
                base: ResourcePathBase::AnalysisRoot,
            }),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                Some(ResourcePath {
                    value: template
                        .quasis
                        .iter()
                        .map(|quasi| {
                            quasi
                                .value
                                .cooked
                                .as_ref()
                                .unwrap_or(&quasi.value.raw)
                                .as_ref()
                        })
                        .collect(),
                    base: ResourcePathBase::AnalysisRoot,
                })
            }
            Expression::NewExpression(new) => self.static_new_module_url(new),
            Expression::CallExpression(call) => self.static_file_url_call(call),
            Expression::StaticMemberExpression(member)
                if member.property.name == "dirname"
                    && matches!(&member.object, Expression::MetaProperty(meta)
                        if meta.meta.name == "import" && meta.property.name == "meta") =>
            {
                Some(ResourcePath {
                    value: ".".to_string(),
                    base: ResourcePathBase::SourceModule,
                })
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                self.static_resource_expression(&parenthesized.expression)
            }
            _ => None,
        }
    }

    fn static_file_url_call(&self, call: &CallExpression<'_>) -> Option<ResourcePath> {
        let file_url_to_path = match &call.callee {
            Expression::Identifier(callee) => {
                matches!(
                    self.binding(callee.name.as_str()),
                    Some(Binding::FileUrlToPath)
                )
            }
            Expression::StaticMemberExpression(member) => {
                matches!(&member.object, Expression::Identifier(namespace)
                    if member.property.name == "fileURLToPath"
                        && matches!(self.binding(namespace.name.as_str()), Some(Binding::UrlNamespace)))
                    || (!self.is_shadowed("require")
                        && inline_require_file_url_to_path(&call.callee))
            }
            _ => false,
        };
        file_url_to_path
            .then(|| self.static_module_url(call.arguments.first()?))
            .flatten()
    }
}
