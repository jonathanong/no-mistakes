use super::*;

impl Builder<'_> {
    pub(super) fn reference(&mut self, path: &Path, name: &str) -> Option<NodeId> {
        let file = self.facts(path);
        if file.parse_error.is_some() || file.operational_error.is_some() {
            return None;
        }
        let mut candidates = file.ancestry.classes.iter().filter(|class| {
            class.module_level
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
        let (head, tail) = name.split_once('.').unwrap_or((name, ""));
        let binding = file
            .imported_bindings
            .iter()
            .find(|binding| binding.local == head && !binding.is_type_only)?;
        let target = self.module(path, &binding.specifier)?;
        let exported = match binding.kind {
            ImportedBindingKind::Namespace => tail.to_string(),
            _ if tail.is_empty() => binding.imported.clone(),
            _ => format!("{}.{}", binding.imported, tail),
        };
        self.exported(&target, &exported)
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
