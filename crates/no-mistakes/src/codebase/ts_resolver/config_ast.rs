fn last_object_property<'ast, 'source>(
    object: &'ast jsonc_parser::ast::Object<'source>,
    name: &str,
) -> Option<&'ast jsonc_parser::ast::Object<'source>> {
    object
        .properties
        .iter()
        .rev()
        .find(|property| property.name.as_str() == name)
        .and_then(|property| property.value.as_object())
}

fn collect_path_mappings(
    paths: &jsonc_parser::ast::Object<'_>,
    content: &str,
    path: &Path,
    locate_paths: bool,
) -> (Vec<(String, Vec<String>)>, Option<TsConfigPathLocations>) {
    let mut mappings: Vec<(String, Vec<String>)> = Vec::new();
    let mut positions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut locations = locate_paths.then(|| TsConfigPathLocations {
        file: path.to_path_buf(),
        lines: std::collections::BTreeMap::new(),
    });
    let newlines: Vec<usize> = if locate_paths {
        content
            .match_indices('\n')
            .map(|(index, _)| index)
            .collect()
    } else {
        Vec::new()
    };
    for property in &paths.properties {
        let replacements = property
            .value
            .as_array()
            .map(|array| {
                array
                    .elements
                    .iter()
                    .filter_map(|value| {
                        value
                            .as_string_lit()
                            .map(|literal| literal.value.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let pattern = property.name.as_str().to_string();
        if let Some(locations) = &mut locations {
            // The AST range identifies the winning key, never a comment or string
            // that happens to contain the same alias. Offsets are UTF-8 bytes.
            let line = newlines.partition_point(|index| *index < property.range.start) + 1;
            locations.lines.insert(pattern.clone(), line);
        }
        if let Some(index) = positions.get(&pattern).copied() {
            mappings[index].1 = replacements;
        } else {
            positions.insert(pattern.clone(), mappings.len());
            mappings.push((pattern, replacements));
        }
    }
    (mappings, locations)
}
