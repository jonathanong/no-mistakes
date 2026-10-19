impl ImportCollector {
    /// `arguments[0].client.get = …` writes the formal at that index. A missing
    /// formal gets one synthetic slot so the call argument is still a receiver.
    fn project_arguments_member(
        &mut self,
        offset: u32,
        path: Option<&[Option<String>]>,
        completed_at: u32,
    ) -> bool {
        let Some((index, tail)) = arguments_index(path) else {
            return false;
        };
        let Some(owner) = self.current_function_id() else {
            return false;
        };
        let found = self
            .value_flow
            .parameters
            .iter()
            .find(|parameter| {
                parameter.owner == owner
                    && parameter.index == index
                    && parameter.projection.is_empty()
            })
            .map(|parameter| (parameter.scope, parameter.name.clone()));
        let (scope, name) =
            found.unwrap_or_else(|| self.synthesize_arguments_parameter(owner, index));
        let dotted = std::iter::once(name.as_str())
            .chain(tail.iter().filter_map(|part| part.as_deref()))
            .collect::<Vec<_>>()
            .join(".");
        self.invalidate_value_write_with_path(scope, &dotted, offset, Some(tail), completed_at);
        true
    }

    fn synthesize_arguments_parameter(
        &mut self,
        owner: CallableId,
        index: usize,
    ) -> (usize, String) {
        let scope = self.current_lexical_scope_id();
        let name = format!("arguments[{index}]");
        let initialization_order = self.value_flow.parameters.len();
        self.value_parameter_indexes
            .entry(scope)
            .or_default()
            .entry(name.clone())
            .or_default()
            .push(initialization_order);
        self.value_flow
            .parameters
            .push(extract_value_flow::ValueParameter {
                owner,
                index,
                name: name.clone(),
                scope,
                default: extract_value_flow::ValueExpression::Unknown,
                projection: Vec::new(),
                initialization_order,
                default_order: initialization_order,
                default_mutations: None,
                invalidated_at: None,
                mutation_contexts: fx_set(),
                type_reference: None,
            });
        (scope, name)
    }
}

fn arguments_index(path: Option<&[Option<String>]>) -> Option<(usize, &[Option<String>])> {
    let (head, tail) = path.and_then(|path| path.split_first())?;
    let index = head.as_deref().and_then(|text| text.parse().ok())?;
    Some((index, tail))
}
