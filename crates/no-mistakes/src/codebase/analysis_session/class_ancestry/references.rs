use super::*;

impl Builder<'_> {
    pub(super) fn reference(&mut self, path: &Path, name: &str) -> Option<NodeId> {
        let key = (path.to_path_buf(), format!("reference:{name}"));
        if !self.active.insert(key.clone()) {
            return None;
        }
        let result = self.reference_inner(path, name);
        self.active.remove(&key);
        result
    }

    fn reference_inner(&mut self, path: &Path, name: &str) -> Option<NodeId> {
        let file = self.facts(path);
        if file.parse_error.is_some() || file.operational_error.is_some() {
            return None;
        }
        let mut candidates = file.ancestry.classes.iter().filter(|class| {
            class.module_level
                && class.module.is_none()
                && class
                    .namespace
                    .iter()
                    .chain(std::iter::once(&class.name))
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join(".")
                    == name
        });
        if let Some(class) = candidates.next() {
            if candidates.next().is_some() || !namespace_exported(&file, class) {
                return None;
            }
            return Some(self.class(path, &file, class));
        }
        let aliases: Vec<_> = file
            .ancestry
            .declaration_aliases
            .iter()
            .filter(|(local, _)| local == name)
            .collect();
        if aliases.len() == 1 {
            return self.reference(path, &aliases[0].1);
        }
        if !aliases.is_empty() {
            return None;
        }
        let aliases: Vec<_> = file
            .callable_aliases
            .iter()
            .filter(|alias| {
                alias.local == name
                    && alias.scope_id.is_none()
                    && alias.binding_scope == 0
                    && alias.invalidated_at.is_none()
            })
            .collect();
        if aliases.len() == 1 {
            let alias = aliases[0];
            if !super::constructor_aliases::unmutated_reference(
                &file,
                &alias.target,
                alias.declared_at,
                alias.scope_id,
            ) {
                return None;
            }
            return self.reference(path, &alias.target);
        }
        if !aliases.is_empty() {
            return None;
        }
        let mut values = file.value_flow.bindings.iter().filter(|binding| {
            binding.name == name && binding.scope == 0 && binding.owner.is_none()
        });
        if let Some(binding) = values.next() {
            if values.next().is_some()
                || !binding.immutable
                || binding.invalidated_at.is_some()
                || !binding.mutation_contexts.is_empty()
            {
                return None;
            }
            let (target, _) = super::constructor_aliases::binding_reference(&file, &binding.value)?;
            if !super::constructor_aliases::unmutated_reference(
                &file,
                &target,
                binding.declared_at,
                binding.owner,
            ) {
                return None;
            }
            return self.reference(path, &target);
        }
        let (head, tail) = name.split_once('.').unwrap_or((name, ""));
        let mut bindings = file
            .imported_bindings
            .iter()
            .filter(|binding| binding.local == head && !binding.is_type_only);
        let binding = bindings.next()?;
        if bindings.next().is_some() {
            return None;
        }
        let exported = match binding.kind {
            ImportedBindingKind::Namespace => tail.to_string(),
            _ if tail.is_empty() => binding.imported.clone(),
            _ => format!("{}.{}", binding.imported, tail),
        };
        if let Some(target) = self.module(path, &binding.specifier) {
            return self.exported(&target, &exported);
        }
        let (target, id) = self
            .declarations
            .ambient_class(&binding.specifier, path, &exported)?;
        let file = self.facts(&target);
        let class = file.ancestry.classes.iter().find(|class| class.id == id)?;
        Some(self.class(&target, &file, class))
    }
}

fn namespace_exported(file: &TsFileFacts, class: &ClassShape) -> bool {
    if class.namespace.is_empty() {
        return true;
    }
    let bodies = &file.ancestry.namespace_bodies;
    if !bodies
        .iter()
        .filter(|body| body.namespace == class.namespace)
        .flat_map(|body| &body.values)
        .any(|value| value.class == Some(class.id) && value.exported)
    {
        return false;
    }
    (1..class.namespace.len()).all(|depth| {
        bodies
            .iter()
            .filter(|body| body.namespace == class.namespace[..depth])
            .flat_map(|body| &body.values)
            .any(|value| value.name == class.namespace[depth] && value.exported)
    })
}
