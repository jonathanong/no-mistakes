use super::*;

impl ValueFlowResolution<'_, '_> {
    pub(super) fn project_parameter(
        &mut self,
        path: &std::path::Path,
        parameter: &ValueParameter,
        mut value: FlowValue,
        from_default: bool,
    ) -> FlowValue {
        let mut origin_order = from_default.then_some(parameter.default_order);
        let mut origin = from_default.then_some(0);
        for (position, step) in parameter.projection.iter().enumerate() {
            value = match step {
                ValueProjection::Element(_)
                    if !matches!(value.executor(), FlowValue::Array { .. }) =>
                {
                    FlowValue::Unknown
                }
                ValueProjection::Member(member) | ValueProjection::Element(member) => {
                    if let Some(order) = origin_order {
                        self.initializing
                            .push((path.to_path_buf(), parameter.owner, order));
                    }
                    let value = self.member(value, member);
                    if origin_order.is_some() {
                        self.initializing.pop();
                    }
                    value
                }
                ValueProjection::Default(fallback, order) if value.is_undefined() => {
                    self.initializing
                        .push((path.to_path_buf(), parameter.owner, *order));
                    let value = self.value(path, fallback);
                    self.initializing.pop();
                    origin_order = Some(*order);
                    origin = Some(position + 1);
                    value
                }
                ValueProjection::Default(_, _) => value,
            };
            if value.is_undefined() {
                self.default_origins = None;
            }
        }
        if let Some(origin) = origin {
            self.record_default_origin(path, parameter, origin);
        }
        value
    }
}
