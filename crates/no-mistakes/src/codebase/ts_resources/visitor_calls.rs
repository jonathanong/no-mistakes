use super::bindings::{
    fs_method, fs_promise_method, glob_method, import_binding, require_module, Binding,
};
use super::bindings_calls::{inline_require_callee, nested_fs_promise_callee};
use super::{ResourceCall, ResourceCallKind, ResourceDiagnosticKind, ResourceVisitor};
use oxc_ast::ast::{CallExpression, Expression};

impl<'a> ResourceVisitor<'a> {
    pub(super) fn record_call(&mut self, call: &CallExpression<'_>) {
        let Some((binding_name, kind)) = self.resolve_callee(&call.callee) else {
            return;
        };
        if self.is_shadowed(binding_name) {
            return;
        }
        let is_glob = matches!(kind, ResourceCallKind::Glob | ResourceCallKind::GlobSync);
        let Some(argument) = call.arguments.first() else {
            self.emit_diagnostic(
                if is_glob {
                    ResourceDiagnosticKind::DynamicPattern
                } else {
                    ResourceDiagnosticKind::DynamicPath
                },
                call.span.start,
            );
            return;
        };
        let Some(path) = self.static_resource_path(argument) else {
            self.emit_diagnostic(
                if is_glob {
                    ResourceDiagnosticKind::DynamicPattern
                } else {
                    ResourceDiagnosticKind::DynamicPath
                },
                call.span.start,
            );
            return;
        };
        let cwd = if is_glob {
            match self.static_glob_cwd(call.arguments.get(1)) {
                Ok(cwd) => cwd,
                Err(()) => {
                    self.emit_diagnostic(ResourceDiagnosticKind::DynamicCwd, call.span.start);
                    return;
                }
            }
        } else {
            None
        };
        self.facts.calls.push(ResourceCall {
            kind,
            path,
            cwd,
            line: crate::codebase::ts_source::byte_offset_to_line(
                self.source,
                call.span.start as usize,
            ) as usize,
            function_scope: self.current_scope(),
        });
    }

    fn resolve_callee<'b>(
        &self,
        callee: &'b Expression<'b>,
    ) -> Option<(&'b str, ResourceCallKind)> {
        match callee {
            Expression::Identifier(id) => match self.binding(id.name.as_str()) {
                Some(Binding::FsMethod(kind) | Binding::GlobMethod(kind)) => {
                    Some((id.name.as_str(), kind))
                }
                _ => None,
            },
            Expression::CallExpression(_) if !self.is_shadowed("require") => {
                let module = require_module(callee)?;
                match import_binding(module, "default") {
                    Some(Binding::GlobMethod(kind)) => Some(("require", kind)),
                    _ => None,
                }
            }
            Expression::StaticMemberExpression(member) => {
                let Expression::Identifier(object) = &member.object else {
                    return self
                        .nested_fs_promise_callee(callee)
                        .or_else(|| self.inline_require_callee(callee));
                };
                let name = object.name.as_str();
                let method = member.property.name.as_str();
                match self.binding(name) {
                    Some(Binding::FsNamespace) => fs_method(method).map(|kind| (name, kind)),
                    Some(Binding::FsPromisesNamespace) => {
                        fs_promise_method(method).map(|kind| (name, kind))
                    }
                    Some(Binding::GlobNamespace) => Some((name, glob_method(method)?)),
                    Some(Binding::GlobMethod(ResourceCallKind::Glob)) => {
                        Some((name, glob_method(method)?))
                    }
                    _ => self.inline_require_callee(callee),
                }
            }
            Expression::ComputedMemberExpression(_) => None,
            Expression::ParenthesizedExpression(inner) => self.resolve_callee(&inner.expression),
            _ => self.nested_fs_promise_callee(callee),
        }
    }

    fn nested_fs_promise_callee<'b>(
        &self,
        callee: &'b Expression<'b>,
    ) -> Option<(&'b str, ResourceCallKind)> {
        nested_fs_promise_callee(callee, &self.current_bindings())
    }

    fn inline_require_callee<'b>(
        &self,
        callee: &'b Expression<'b>,
    ) -> Option<(&'b str, ResourceCallKind)> {
        (!self.is_shadowed("require"))
            .then(|| inline_require_callee(callee))
            .flatten()
    }
}
