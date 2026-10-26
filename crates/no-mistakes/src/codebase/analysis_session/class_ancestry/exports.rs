use super::*;

enum ClassExport {
    Class(NodeId),
    Absent,
    Unknown,
}

impl Builder<'_> {
    pub(super) fn exported(&mut self, path: &Path, name: &str) -> Option<NodeId> {
        match self.exported_resolution(path, name) {
            ClassExport::Class(target) => Some(target),
            ClassExport::Absent | ClassExport::Unknown => None,
        }
    }

    fn exported_resolution(&mut self, path: &Path, name: &str) -> ClassExport {
        let key = (path.to_path_buf(), name.to_string());
        if !self.active.insert(key.clone()) {
            return ClassExport::Unknown;
        }
        let result = self.exported_inner(path, name);
        self.active.remove(&key);
        result
    }

    fn exported_inner(&mut self, path: &Path, name: &str) -> ClassExport {
        let file = self.facts(path);
        if file.parse_error.is_some() || file.operational_error.is_some() {
            return ClassExport::Unknown;
        }
        if let Some(namespace) = &file.ancestry.export_assignment {
            let local = if name == "default" {
                namespace.clone()
            } else {
                format!(
                    "{namespace}.{}",
                    name.strip_prefix("default.").unwrap_or(name)
                )
            };
            return self
                .reference(path, &local)
                .map_or(ClassExport::Unknown, ClassExport::Class);
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
                Some(specifier) => match self.module(path, specifier) {
                    Some(target) => match self.exported_resolution(&target, &local) {
                        ClassExport::Absent => ClassExport::Unknown,
                        result => result,
                    },
                    None => ClassExport::Unknown,
                },
                None => self
                    .reference(path, &local)
                    .map_or(ClassExport::Unknown, ClassExport::Class),
            };
        }
        if !exports.is_empty() {
            return ClassExport::Unknown;
        }
        // `export *` never forwards a default, including `default.Member`.
        if head == "default" {
            return ClassExport::Absent;
        }
        let mut candidates = Vec::new();
        for specifier in &file.star_reexport_specifiers {
            let Some(target) = self.module(path, specifier) else {
                return ClassExport::Unknown;
            };
            match self.exported_resolution(&target, name) {
                ClassExport::Class(target) => candidates.push(target),
                // A resolved module exporting another name does not make this
                // star ambiguous; opaque/missing modules still do.
                ClassExport::Absent => {}
                ClassExport::Unknown => return ClassExport::Unknown,
            }
        }
        candidates.sort();
        candidates.dedup();
        match candidates.len() {
            0 => ClassExport::Absent,
            1 => ClassExport::Class(candidates.remove(0)),
            _ => ClassExport::Unknown,
        }
    }
}
