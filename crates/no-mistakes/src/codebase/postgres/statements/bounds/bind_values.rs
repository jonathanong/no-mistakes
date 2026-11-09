use super::Scope;
use std::{collections::BTreeMap, sync::Arc};
pub(crate) type BindValues = Arc<BTreeMap<(u32, u32), u32>>;

impl Scope {
    pub(in crate::codebase::postgres) fn with_bind_values(mut self, values: BindValues) -> Self {
        self.bind_values = values;
        self
    }
    pub(in crate::codebase::postgres) fn bind_values(&self) -> BindValues {
        Arc::clone(&self.bind_values)
    }
}
