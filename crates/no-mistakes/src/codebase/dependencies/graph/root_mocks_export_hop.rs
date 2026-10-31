fn owned_hop(facts: &TsFileFacts, export: &str) -> Hop {
    let mut bindings = facts
        .exported_bindings
        .iter()
        .filter(|binding| binding.exported == export);
    let Some(binding) = bindings.next() else {
        return if facts.star_reexport_specifiers.is_empty() {
            Hop::Miss
        } else {
            Hop::Stars(facts.star_reexport_specifiers.clone())
        };
    };
    if bindings.next().is_some() {
        return Hop::Uncertain;
    }
    if let Some(specifier) = binding.specifier.clone() {
        return Hop::Next(specifier, binding.local.clone());
    }
    if facts
        .known_function_scopes
        .iter()
        .any(|scope| scope == &binding.local)
        || facts
            .class_scopes
            .iter()
            .any(|scope| scope == &binding.local)
    {
        return Hop::Miss;
    }
    let mut imports = facts
        .imported_bindings
        .iter()
        .filter(|import| !import.is_type_only && import.local == binding.local);
    let Some(import) = imports.next() else {
        return Hop::Miss;
    };
    if imports.next().is_some() {
        return Hop::Uncertain;
    }
    Hop::Next(import.specifier.clone(), import.imported.clone())
}
