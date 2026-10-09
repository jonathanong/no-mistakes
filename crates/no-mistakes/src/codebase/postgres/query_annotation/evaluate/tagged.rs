use super::{concat, Environment, Evaluator, Value};
use crate::codebase::postgres::query_annotation::Expr;
use std::path::{Path, PathBuf};

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn tag_trusted(&self, tag: &str, path: &Path, env: Environment) -> bool {
        let file = &self.files[path];
        let name = if tag == "String.raw" { "String" } else { tag };
        let trusted = if tag == "String.raw" {
            !file.facts.raw_tag_reassigned && !file.imports.contains_key(name)
        } else {
            file.facts.trusted_tags.contains(tag)
        };
        trusted && !self.scopes[env].contains_key(name) && !file.facts.globals.contains_key(name)
    }
    pub(super) fn tagged(
        &mut self,
        tag: &str,
        parts: &[Expr],
        effects: &[Expr],
        path: &Path,
        env: &Environment,
        context: (u8, bool),
    ) -> Value {
        let (depth, generic) = context;
        let values = effects
            .iter()
            .map(|effect| self.expr(effect, path, env, depth, generic))
            .collect::<Vec<_>>();
        let mut prefix = Value::Prefix(String::new(), true, None);
        for part in parts {
            prefix = concat(prefix, self.expr(part, path, env, depth, generic));
        }
        let trusted = self.tag_trusted(tag, path, *env);
        let name = if tag == "String.raw" { "String" } else { tag };
        let legacy = self.files[path].facts.legacy_tag_spans.get(name).is_some_and(|start| {
            let local = self.scopes[*env].get(name);
            local.is_none() || matches!(local, Some(Value::Function(function, owner, _)) if owner == path && function.start == *start)
        });
        if !trusted && !legacy {
            self.invalidate_builders(&values);
            if let Some(Value::Function(function, _, captured)) = values.first() {
                if !function.supported {
                    self.invalidate_captured(*captured, function);
                }
            }
        }
        if trusted || legacy {
            if let Value::Prefix(_, _, id) = &mut prefix {
                if tag != "String.raw" || legacy {
                    *id = Some(self.next_builder);
                    self.next_builder += 1;
                }
            }
            // Reuse the existing local-tag contract only for this annotation
            // prefix; no complete-SQL fact is promoted for structural rules.
            if legacy
                && matches!(values.first(), Some(Value::Function(function, _, _)) if function.asynchronous)
            {
                Value::Promise(Box::new(prefix))
            } else {
                prefix
            }
        } else {
            Value::Unknown
        }
    }
}
