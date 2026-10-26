fn normalize_input(mut value: Input, syntax: &Syntax, owners: &Owners<'_>) -> Input {
    if let Input::Reference(reference) = value {
        value = owners
            .get(&reference)
            .filter(|binding| !binding.imported && binding.runtime)
            .and_then(|binding| {
                syntax
                    .constants
                    .get(&(binding.span.start, binding.span.end))
            })
            .cloned()
            .unwrap_or(Input::Unknown);
    }
    if let Input::Sql { holes, .. } = &mut value {
        for (_, hole) in holes {
            if let Some(key) = hole {
                let reference = syntax.env_references[key];
                if owners.contains_key(&reference) {
                    *hole = None;
                } else {
                    *key = key.rsplit_once('@').unwrap().0.to_string();
                }
            }
        }
    }
    value
}
