use super::{view_definition::ViewProjections, SchemaCatalog};
use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

#[derive(Debug, Default)]
pub(super) struct ProjectionCache {
    pub(super) scalar_function_names: OnceLock<Arc<BTreeSet<Vec<String>>>>,
    pub(super) view_projections: OnceLock<ViewProjections>,
}

impl SchemaCatalog {
    pub(super) fn projections(&self) -> &ProjectionCache {
        self.projections.get_or_init(Default::default)
    }
}
