use super::evaluate::{Evaluator, Value};
use super::{expressions, Expr, Step};
use crate::fx::{fx_set, FxHashSet};
use oxc_ast::ast::{ExportDefaultDeclaration, ExportDefaultDeclarationKind};
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

// ESM ambiguity depends on the originating binding, not the number of paths.
struct ResolvedExport {
    identity: Option<(PathBuf, String)>,
    value: Value,
}

impl ResolvedExport {
    fn unknown() -> Self {
        Self {
            identity: None,
            value: Value::Unknown,
        }
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
        self.lookup_export_inner(path, name, depth.min(16), generic, &mut fx_set())
            .map(|provider| provider.value)
    }

    fn lookup_export_inner(
        &mut self,
        path: &Path,
        name: &str,
        depth: u8,
        generic: bool,
        visiting: &mut FxHashSet<(PathBuf, String)>,
    ) -> Option<ResolvedExport> {
        let Some(depth) = depth.checked_sub(1) else {
            return Some(ResolvedExport::unknown());
        };
        let key = (path.to_path_buf(), name.to_string());
        if !visiting.insert(key.clone()) {
            return Some(ResolvedExport::unknown());
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
        visiting: &mut FxHashSet<(PathBuf, String)>,
    ) -> Option<ResolvedExport> {
        let Some(file) = self.files.get(path) else {
            return Some(ResolvedExport::unknown());
        };
        let binding = file
            .exports
            .get(name)
            .map(|index| file.ts.exported_bindings[*index].clone());
        let stars = file.ts.star_reexport_specifiers.clone();
        if let Some(binding) = binding {
            let import = (!file.facts.globals.contains_key(&binding.local))
                .then(|| file.imports.get(&binding.local))
                .flatten()
                .map(|index| file.ts.imported_bindings[*index].clone());
            let forwarded = binding
                .specifier
                .map(|specifier| (specifier, binding.local.clone()))
                .or_else(|| import.map(|binding| (binding.specifier, binding.imported)));
            if let Some((specifier, name)) = forwarded {
                return Some(match (self.resolve)(&specifier, path) {
                    Some(target) => self
                        .lookup_export_inner(&target, &name, depth, generic, visiting)
                        .unwrap_or_else(ResolvedExport::unknown),
                    None => ResolvedExport::unknown(),
                });
            }
            return Some(ResolvedExport {
                identity: Some((path.to_path_buf(), binding.local.clone())),
                value: self.name(path, &binding.local, depth, generic),
            });
        }
        // Default is never forwarded by stars; explicit bindings take precedence.
        if name == "default" {
            return None;
        }
        let mut provider: Option<ResolvedExport> = None;
        for specifier in stars {
            let value = match (self.resolve)(&specifier, path) {
                Some(target) => self.lookup_export_inner(&target, name, depth, generic, visiting),
                None => Some(ResolvedExport::unknown()),
            };
            if let Some(value) = value {
                if let Some(existing) = &provider {
                    if existing.identity.is_none() || existing.identity != value.identity {
                        return Some(ResolvedExport::unknown());
                    }
                } else {
                    provider = Some(value);
                }
            }
        }
        provider
    }
}
