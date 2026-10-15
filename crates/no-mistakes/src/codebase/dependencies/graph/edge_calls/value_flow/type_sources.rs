use super::*;
use crate::codebase::dependencies::extract::ValueTypeSource;
use type_scopes::AnnotationScope;
type Seen = FxHashSet<(std::path::PathBuf, String, Vec<String>, usize)>;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn annotation_origin(
        &self,
        file: &std::path::Path,
        source: &ValueTypeSource,
    ) -> Option<(String, String)> {
        self.annotation_projection(file, source, &[], AnnotationScope::default(), &mut fx_set())
    }
    pub(super) fn annotation_projection(
        &self,
        file: &std::path::Path,
        source: &ValueTypeSource,
        members: &[String],
        scope: AnnotationScope<'_>,
        seen: &mut Seen,
    ) -> Option<(String, String)> {
        match source {
            ValueTypeSource::Scoped {
                scope: lexical,
                source,
            } => self.annotation_projection(
                file,
                source,
                members,
                AnnotationScope {
                    lexical: *lexical,
                    ..scope
                },
                seen,
            ),
            ValueTypeSource::NonNullable(source) => self
                .global_non_nullable(file, scope.lexical)?
                .then(|| self.annotation_projection(file, source, members, scope, seen))
                .flatten(),
            ValueTypeSource::Indexed(base, member) => {
                let mut path = vec![member.clone()];
                path.extend_from_slice(members);
                self.annotation_projection(file, base, &path, scope, seen)
            }
            ValueTypeSource::Object(fields) => {
                let (member, rest) = members.split_first()?;
                self.annotation_projection(file, fields.get(member)?, rest, scope, seen)
            }
            ValueTypeSource::Reference(reference) => {
                self.annotation_reference(file, reference, members, scope, seen)
            }
            ValueTypeSource::Function(_) => None,
        }
    }
    fn annotation_reference(
        &self,
        file: &std::path::Path,
        reference: &str,
        members: &[String],
        scope: AnnotationScope<'_>,
        seen: &mut Seen,
    ) -> Option<(String, String)> {
        if !seen.insert((
            file.to_path_buf(),
            reference.to_owned(),
            members.to_vec(),
            scope.lexical,
        )) {
            return None;
        }
        let facts = self.facts.get_ts_facts(file)?;
        let (root, namespace) = reference
            .split_once('.')
            .map_or((reference, None), |(root, member)| (root, Some(member)));
        let local = self.annotation_binding_scope(file, scope.lexical, root)?;
        if local.is_some_and(|scope| scope != 0) {
            return None;
        }
        if let Some(imported) = facts
            .imported_bindings
            .iter()
            .find(|binding| binding.local == root)
            .filter(|_| local.is_none())
        {
            let export = super::provenance::type_export(imported, namespace)?;
            return if let Some((member, rest)) = members.split_first() {
                self.imported_annotation_projection(
                    file,
                    &imported.specifier,
                    &export,
                    (member, rest),
                    seen,
                )
            } else {
                Some(self.type_origin(file, imported.specifier.clone(), export))
            };
        }
        if members.is_empty() && namespace.is_none() {
            let identity = facts
                .value_flow
                .type_exports
                .iter()
                .find(|binding| binding.local == root && binding.specifier.is_none())?;
            return Some((scope.module?.to_owned(), identity.exported.clone()));
        }
        let fields = facts
            .value_flow
            .type_sources
            .as_ref()?
            .declarations
            .get(reference)?;
        let (member, rest) = members.split_first()?;
        self.annotation_projection(
            file,
            fields.get(member)?,
            rest,
            AnnotationScope {
                lexical: 0,
                ..scope
            },
            seen,
        )
    }
    fn imported_annotation_projection(
        &self,
        file: &std::path::Path,
        module: &str,
        export: &str,
        path: (&str, &[String]),
        seen: &mut Seen,
    ) -> Option<(String, String)> {
        let target = self.resolver.resolve(module, file)?;
        let target = self.inputs.graph_files.visible_path(&target)?;
        let facts = self.facts.get_ts_facts(target)?;
        let binding = facts
            .value_flow
            .type_exports
            .iter()
            .find(|binding| binding.exported == export)?;
        let (member, rest) = path;
        if let Some(specifier) = &binding.specifier {
            let mut members = vec![member.to_owned()];
            members.extend_from_slice(rest);
            if !seen.insert((target.to_path_buf(), format!("export:{export}"), members, 0)) {
                return None;
            }
            return self.imported_annotation_projection(
                target,
                specifier,
                &binding.local,
                path,
                seen,
            );
        }
        let fields = facts
            .value_flow
            .type_sources
            .as_ref()?
            .declarations
            .get(&binding.local)?;
        self.annotation_projection(
            target,
            fields.get(member)?,
            rest,
            AnnotationScope {
                lexical: 0,
                module: Some(module),
            },
            seen,
        )
    }
}
