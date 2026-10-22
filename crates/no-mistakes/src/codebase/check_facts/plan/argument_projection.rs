impl CheckFactPlan {
    #[doc(hidden)]
    pub fn require_call_arguments(
        &mut self,
        projection: Option<crate::codebase::ts_source::facts::ArgumentProjection>,
    ) {
        let Some(projection) = projection else {
            return;
        };
        if self.graph.call_arguments {
            self.graph_context.argument_projection.include(&projection);
        } else {
            self.graph_context.argument_projection = projection;
            self.graph.call_arguments = true;
        }
    }

    fn merged_argument_projection(
        &self,
        other: &Self,
    ) -> crate::codebase::ts_source::facts::ArgumentProjection {
        if self.graph.call_arguments && other.graph.call_arguments {
            let mut scope = self.graph_context.argument_projection.clone();
            scope.include(&other.graph_context.argument_projection);
            scope
        } else if other.graph.call_arguments {
            other.graph_context.argument_projection.clone()
        } else {
            self.graph_context.argument_projection.clone()
        }
    }
}
