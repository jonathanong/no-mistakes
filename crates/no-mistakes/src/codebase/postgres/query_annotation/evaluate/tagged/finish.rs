use super::*;

impl<F: Fn(&str, &Path) -> Option<PathBuf>> Evaluator<'_, F> {
    pub(super) fn finish_tagged(
        &mut self,
        tag: &str,
        path: &Path,
        env: Environment,
        values: &[Value],
        depth: u8,
        mut prefix: Value,
    ) -> Value {
        let trusted = self.tag_trusted(tag, path, env);
        let name = if tag == "String.raw" { "String" } else { tag };
        let legacy = self.files[path].facts.legacy_tag_spans.get(name).is_some_and(|start| {
            let local = self.scopes[env].get(name);
            local.is_none() || matches!(local, Some(Value::Function(function, owner, _)) if owner.as_path() == path && function.start == *start)
        });
        if !trusted && !legacy {
            self.invalidate_opaque_builders(values);
            self.opaque_callbacks(values, depth);
        }
        if trusted || legacy {
            if let Value::Prefix(_, _, id) = &mut prefix {
                if tag != "String.raw" || legacy {
                    let builder = self.next_builder;
                    *id = Some(builder);
                    self.next_builder += 1;
                    if trusted {
                        self.record_builder_tag(builder, tag, path);
                    }
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
