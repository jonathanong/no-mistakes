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
struct ResolvedExport<'a> {
    identity: Option<(&'a Path, &'a str)>,
    value: Value,
}

impl ResolvedExport<'_> {
    fn unknown() -> Self {
        Self {
            identity: None,
            value: Value::Unknown,
        }
    }
}

impl<'a, F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'a, F> {
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

    fn lookup_export_inner<'b>(
        &mut self,
        path: &'b Path,
        name: &'b str,
        depth: u8,
        generic: bool,
        visiting: &mut FxHashSet<(&'b Path, &'b str)>,
    ) -> Option<ResolvedExport<'a>>
    where
        'a: 'b,
    {
        let Some(depth) = depth.checked_sub(1) else {
            return Some(ResolvedExport::unknown());
        };
        let key = (path, name);
        if !visiting.insert(key) {
            return Some(ResolvedExport::unknown());
        }
        let value = self.lookup_export_provider(path, name, depth, generic, visiting);
        visiting.remove(&key);
        value
    }

    fn lookup_export_provider<'b>(
        &mut self,
        path: &'b Path,
        name: &'b str,
        depth: u8,
        generic: bool,
        visiting: &mut FxHashSet<(&'b Path, &'b str)>,
    ) -> Option<ResolvedExport<'a>>
    where
        'a: 'b,
    {
        let files = self.files;
        let Some((origin, file)) = files.get_key_value(path) else {
            return Some(ResolvedExport::unknown());
        };
        let binding = file
            .module
            .exports
            .get(name)
            .map(|index| &file.module.ts.exported_bindings[*index]);
        let stars = &file.module.ts.star_reexport_specifiers;
        if let Some(binding) = binding {
            let import = (!file.facts.globals.contains_key(&binding.local))
                .then(|| file.module.imports.get(binding.local.as_str()))
                .flatten()
                .map(|index| &file.module.ts.imported_bindings[*index]);
            let forwarded = binding
                .specifier
                .as_deref()
                .map(|specifier| (specifier, binding.local.as_str()))
                .or_else(|| {
                    import.map(|binding| (binding.specifier.as_str(), binding.imported.as_str()))
                });
            if let Some((specifier, name)) = forwarded {
                return Some(match file.module.target(specifier, path, self.resolver()) {
                    Some(target) => self
                        .lookup_export_inner(target, name, depth, generic, visiting)
                        .unwrap_or_else(ResolvedExport::unknown),
                    None => ResolvedExport::unknown(),
                });
            }
            return Some(ResolvedExport {
                identity: Some((origin.as_path(), binding.local.as_str())),
                value: self.name(path, &binding.local, depth, generic),
            });
        }
        // Default is never forwarded by stars; explicit bindings take precedence.
        if name == "default" {
            return None;
        }
        let mut provider: Option<ResolvedExport<'a>> = None;
        for specifier in stars {
            let value = match file.module.target(specifier, path, self.resolver()) {
                Some(target) => self.lookup_export_inner(target, name, depth, generic, visiting),
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
