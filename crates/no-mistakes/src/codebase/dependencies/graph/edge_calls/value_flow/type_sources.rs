use super::*;
use crate::codebase::dependencies::extract::ValueTypeSource;
use type_scopes::AnnotationScope;
#[path = "type_references.rs"]
mod references;
#[path = "type_views.rs"]
mod views;
use views::{AnnotationView, TypeQueryGuards};

impl ValueFlowResolution<'_, '_> {
    pub(super) fn annotation_origin(
        &self,
        file: &std::path::Path,
        source: &ValueTypeSource,
    ) -> Option<(String, String)> {
        self.annotation_origin_scoped(file, source, AnnotationScope::default())
    }
    pub(super) fn annotation_origin_scoped<'q>(
        &'q self,
        file: &'q std::path::Path,
        source: &'q ValueTypeSource,
        scope: AnnotationScope<'q>,
    ) -> Option<(String, String)> {
        self.annotation_view(file, source, scope, &mut TypeQueryGuards::default())?
            .origin()
    }
    fn annotation_view<'q>(
        &'q self,
        file: &'q std::path::Path,
        source: &'q ValueTypeSource,
        scope: AnnotationScope<'q>,
        guards: &mut TypeQueryGuards<'q>,
    ) -> Option<AnnotationView<'q>> {
        let key = (file, scope.lexical, source as *const ValueTypeSource);
        if !guards.descriptors.insert(key) {
            return None;
        }
        // Normalize a descriptor before selecting the next member. Returning a
        // lazy object releases this guard, so finite recursive paths can repeat.
        let result = self.annotation_view_inner(file, source, scope, guards);
        guards.descriptors.remove(&key);
        result
    }
    fn annotation_view_inner<'q>(
        &'q self,
        file: &'q std::path::Path,
        source: &'q ValueTypeSource,
        scope: AnnotationScope<'q>,
        guards: &mut TypeQueryGuards<'q>,
    ) -> Option<AnnotationView<'q>> {
        match source {
            ValueTypeSource::Scoped {
                scope: lexical,
                source,
            } => self.annotation_view(
                file,
                source,
                AnnotationScope {
                    lexical: *lexical,
                    ..scope
                },
                guards,
            ),
            ValueTypeSource::NonNullable(source) => self
                .global_non_nullable(file, scope.lexical)?
                .then(|| self.annotation_view(file, source, scope, guards))
                .flatten(),
            ValueTypeSource::Indexed(base, member) => {
                let base = self.annotation_view(file, base, scope, guards)?;
                let AnnotationView::Object {
                    file,
                    fields,
                    scope,
                    ..
                } = base
                else {
                    return None;
                };
                self.annotation_view(file, fields.get(member)?, scope, guards)
            }
            ValueTypeSource::Object(fields) => Some(AnnotationView::Object {
                file,
                fields,
                scope,
                identity: None,
            }),
            ValueTypeSource::Reference(reference) => {
                self.annotation_reference_view(file, reference, scope, guards)
            }
            ValueTypeSource::Function(_) => None,
        }
    }
}
