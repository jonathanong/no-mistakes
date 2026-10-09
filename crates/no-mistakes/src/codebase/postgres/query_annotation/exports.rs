use super::evaluate::{Evaluator, Value};
use super::{expressions, Expr, Step};
use oxc_ast::ast::{ExportDefaultDeclaration, ExportDefaultDeclarationKind};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// Mirror the shared import/export facts' local identity: named declarations
// retain their identifier, while expression and anonymous defaults use default.
pub(super) fn default_steps(export: &ExportDefaultDeclaration<'_>, source: &str) -> Vec<Step> {
    match &export.declaration {
        ExportDefaultDeclarationKind::Identifier(_) => Vec::new(),
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => vec![Step::Hoisted(
            function
                .id
                .as_ref()
                .map_or("default", |id| id.name.as_str())
                .to_string(),
            expressions::function_expression(function, source),
        )],
        other => vec![Step::Bind(
            "default".into(),
            other.as_expression().map_or(Expr::Unsupported, |expr| {
                expressions::expression(expr, source)
            }),
        )],
    }
}

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn lookup_export(
        &mut self,
        path: &Path,
        name: &str,
        depth: u8,
        generic: bool,
    ) -> Option<Value> {
        self.lookup_export_inner(path, name, depth.min(16), generic, &mut BTreeSet::new())
    }

    fn lookup_export_inner(
        &mut self,
        path: &Path,
        name: &str,
        depth: u8,
        generic: bool,
        visiting: &mut BTreeSet<(PathBuf, String)>,
    ) -> Option<Value> {
        let Some(depth) = depth.checked_sub(1) else {
            return Some(Value::Unknown);
        };
        let key = (path.to_path_buf(), name.to_string());
        if !visiting.insert(key.clone()) {
            return Some(Value::Unknown);
        }
        let value = self.lookup_export_provider(path, name, depth, generic, visiting);
        visiting.remove(&key);
        value
    }

    fn lookup_export_provider(
        &mut self,
        path: &Path,
        name: &str,
        depth: u8,
        generic: bool,
        visiting: &mut BTreeSet<(PathBuf, String)>,
    ) -> Option<Value> {
        let Some(file) = self.files.get(path) else {
            return Some(Value::Unknown);
        };
        let binding = file
            .ts
            .exported_bindings
            .iter()
            .find(|binding| binding.exported == name)
            .cloned();
        let stars = file.ts.star_reexport_specifiers.clone();
        if let Some(binding) = binding {
            return Some(if let Some(specifier) = binding.specifier {
                match (self.resolve)(&specifier, path) {
                    Some(target) => self
                        .lookup_export_inner(&target, &binding.local, depth, generic, visiting)
                        .unwrap_or(Value::Unknown),
                    None => Value::Unknown,
                }
            } else {
                self.name(path, &binding.local, depth, generic)
            });
        }
        // ESM star exports never forward a default binding. Explicit bindings
        // above take precedence; multiple star providers remain conservative.
        if name == "default" {
            return None;
        }
        let mut provider = None;
        for specifier in stars {
            let value = match (self.resolve)(&specifier, path) {
                Some(target) => self.lookup_export_inner(&target, name, depth, generic, visiting),
                None => Some(Value::Unknown),
            };
            if let Some(value) = value {
                if provider.is_some() {
                    return Some(Value::Unknown);
                }
                provider = Some(value);
            }
        }
        provider
    }
}
