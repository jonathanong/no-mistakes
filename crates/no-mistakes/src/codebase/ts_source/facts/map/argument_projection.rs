use super::super::{ArgumentProjection, TsFactMap};

impl TsFactMap {
    pub(crate) fn with_argument_projection(mut self, projection: &ArgumentProjection) -> Self {
        if self.plan.call_arguments {
            self.argument_projection = Some(projection.clone());
        }
        self
    }

    pub(crate) fn covers_argument_projection(&self, required: &ArgumentProjection) -> bool {
        self.argument_projection
            .as_ref()
            .is_some_and(|collected| collected.covers(required))
    }
}

#[cfg(test)]
mod tests;
