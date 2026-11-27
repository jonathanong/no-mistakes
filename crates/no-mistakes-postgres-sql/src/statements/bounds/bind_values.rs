use super::Scope;
use std::{collections::BTreeMap, sync::Arc};
pub type BindValues = Arc<BTreeMap<(u32, u32), u32>>;

impl Scope {
    pub(crate) fn with_bind_values(mut self, values: BindValues) -> Self {
        self.bind_values = values;
        self
    }
    pub(crate) fn bind_values(&self) -> BindValues {
        Arc::clone(&self.bind_values)
    }
}
