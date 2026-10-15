use super::*;

#[derive(Default)]
pub(super) struct TypeQueryGuards<'q> {
    pub(super) descriptors: FxHashSet<(&'q std::path::Path, usize, *const ValueTypeSource)>,
    pub(super) exports: FxHashSet<(&'q std::path::Path, String)>,
}

pub(super) enum AnnotationView<'q> {
    Object {
        file: &'q std::path::Path,
        fields: &'q FxHashMap<String, ValueTypeSource>,
        scope: AnnotationScope<'q>,
        identity: Option<(String, String)>,
    },
    Origin(String, String),
}
impl AnnotationView<'_> {
    pub(super) fn origin(self) -> Option<(String, String)> {
        match self {
            Self::Object { identity, .. } => identity,
            Self::Origin(module, export) => Some((module, export)),
        }
    }
}
