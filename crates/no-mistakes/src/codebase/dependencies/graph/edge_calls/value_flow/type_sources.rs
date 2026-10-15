use super::*;
use crate::codebase::dependencies::extract::ValueTypeSource;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn annotation_origin(&self, file: &std::path::Path, source: &ValueTypeSource) -> Option<(String, String)> {
        self.annotation_projection(file, source, &[], &mut fx_set())
    }

    fn annotation_projection(&self, file: &std::path::Path, source: &ValueTypeSource, members: &[String], seen: &mut FxHashSet<(std::path::PathBuf, String, Vec<String>)>) -> Option<(String, String)> {
        match source {
            ValueTypeSource::Indexed(base, member) => {
                let mut path = vec![member.clone()];
                path.extend_from_slice(members);
                self.annotation_projection(file, base, &path, seen)
            }
            ValueTypeSource::Object(fields) => {
                let (member, rest) = members.split_first()?;
                self.annotation_projection(file, fields.get(member)?, rest, seen)
            }
            ValueTypeSource::Reference(reference) => {
                if !seen.insert((file.to_path_buf(), reference.clone(), members.to_vec())) { return None; }
                let facts = self.facts.get_ts_facts(file)?;
                let (root, namespace) = reference.split_once('.').map_or((reference.as_str(), None), |(root, member)| (root, Some(member)));
                if let Some(imported) = facts.imported_bindings.iter().find(|binding| binding.local == root) {
                    let export = super::provenance::type_export(imported, namespace)?;
                    if members.is_empty() {
                        return Some(self.type_origin(file, imported.specifier.clone(), export));
                    }
                    self.imported_annotation_projection(file, &imported.specifier, &export, members, seen)
                } else {
                    let source = facts.value_flow.type_sources.as_ref()?.declarations.get(reference)?;
                    self.annotation_projection(file, source, members, seen)
                }
            }
            ValueTypeSource::Function(_) => None,
        }
    }

    fn imported_annotation_projection(&self, file: &std::path::Path, module: &str, export: &str, members: &[String], seen: &mut FxHashSet<(std::path::PathBuf, String, Vec<String>)>) -> Option<(String, String)> {
        let target = self.resolver.resolve(module, file)?;
        let target = self.inputs.graph_files.visible_path(&target)?;
        let facts = self.facts.get_ts_facts(target)?;
        let binding = facts.value_flow.type_exports.iter().find(|binding| binding.exported == export)?;
        if let Some(specifier) = &binding.specifier {
            if !seen.insert((target.to_path_buf(), format!("export:{export}"), members.to_vec())) { return None; }
            return self.imported_annotation_projection(target, specifier, &binding.local, members, seen);
        }
        let source = facts.value_flow.type_sources.as_ref()?.declarations.get(&binding.local)?;
        let (member, rest) = members.split_first()?;
        let ValueTypeSource::Object(fields) = source else { return None; };
        let source = fields.get(member)?;
        if let ValueTypeSource::Reference(reference) = source {
            // A declaration's own local type identity belongs to this module,
            // rather than to an identically spelled type in the requesting file.
            if rest.is_empty() && !facts.imported_bindings.iter().any(|binding| binding.local == *reference) {
                let identity = facts.value_flow.type_exports.iter().find(|binding| binding.local == *reference && binding.specifier.is_none())?;
                return Some((module.to_owned(), identity.exported.clone()));
            }
        }
        self.annotation_projection(target, source, rest, seen)
    }
}
