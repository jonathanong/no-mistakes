use super::*;

impl Builder<'_> {
    pub(super) fn exported(&mut self, path: &Path, name: &str) -> Option<NodeId> {
        let key = (path.to_path_buf(), name.to_string());
        if !self.active.insert(key.clone()) {
            return None;
        }
        let result = self.exported_inner(path, name);
        self.active.remove(&key);
        result
    }

    fn exported_inner(&mut self, path: &Path, name: &str) -> Option<NodeId> {
        let file = self.facts(path);
        if file.parse_error.is_some() || file.operational_error.is_some() {
            return None;
        }
        if let Some(namespace) = &file.ancestry.export_assignment {
            if name == "default" {
                return self.reference(path, namespace);
            }
            let member = name.strip_prefix("default.").unwrap_or(name);
            return self.reference(path, &format!("{namespace}.{member}"));
        }
        let (head, tail) = name.split_once('.').unwrap_or((name, ""));
        let exports: Vec<_> = file
            .exported_bindings
            .iter()
            .filter(|binding| binding.exported == head)
            .collect();
        if exports.len() == 1 {
            let binding = exports[0];
            let local = if tail.is_empty() {
                binding.local.clone()
            } else {
                format!("{}.{tail}", binding.local)
            };
            return match &binding.specifier {
                Some(specifier) => {
                    let target = self.module(path, specifier)?;
                    self.exported(&target, &local)
                }
                None => self.reference(path, &local),
            };
        }
        if !exports.is_empty() {
            return None;
        }
        let mut candidates = Vec::new();
        for specifier in &file.star_reexport_specifiers {
            let target = self
                .module(path, specifier)
                .and_then(|target| self.exported(&target, name))?;
            candidates.push(target);
        }
        candidates.sort();
        candidates.dedup();
        (candidates.len() == 1).then(|| candidates.remove(0))
    }
}
