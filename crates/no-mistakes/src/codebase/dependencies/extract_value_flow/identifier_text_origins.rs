use super::*;

impl TextOrigins {
    pub(crate) fn identifier_scalars(&self) -> Option<Self> {
        let mut bindings: BindingOrigins = FxHashMap::default();
        for (scope, names) in &self.bindings {
            for (name, declarations) in names {
                for (at, origin) in declarations {
                    if let Some(origin) = scalar(origin) {
                        bindings
                            .entry(*scope)
                            .or_default()
                            .entry(name.clone())
                            .or_default()
                            .insert(*at, origin);
                    }
                }
            }
        }
        let arguments: FxHashMap<_, _> = self
            .arguments
            .iter()
            .filter_map(|(span, args)| {
                let origin = args.first()?.as_ref()?;
                if origin.parts.is_empty() {
                    return None;
                }
                Some((*span, vec![Some(scalar(origin)?)]))
            })
            .collect();
        (!bindings.is_empty() || !arguments.is_empty()).then_some(Self {
            arguments,
            bindings,
        })
    }
}

fn scalar(origin: &TextOrigin) -> Option<TextOrigin> {
    if origin.literal {
        return Some(TextOrigin {
            literal: true,
            ..Default::default()
        });
    }
    retained_parts(
        origin
            .parts
            .iter()
            .map(|part| part.as_ref().and_then(scalar))
            .collect(),
    )
}
