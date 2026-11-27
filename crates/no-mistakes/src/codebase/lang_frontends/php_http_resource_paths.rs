pub fn expand_actions(path: &str, handler: &str, actions: &[&str]) -> Vec<(String, String)> {
    let Some((collection, param)) = collection_param(path) else {
        return Vec::new();
    };
    actions
        .iter()
        .map(|action| {
            (
                action_path(&collection, &param, action),
                format!("{handler}.{action}"),
            )
        })
        .collect()
}

fn collection_param(path: &str) -> Option<(String, String)> {
    let trimmed = path.trim().trim_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let mut prefix = String::new();
    let names: Vec<&str> = trimmed.split('.').collect();
    for (index, name) in names.iter().enumerate() {
        let segment = name.rsplit('/').next().filter(|part| !part.is_empty())?;
        if index + 1 == names.len() {
            let collection = if prefix.is_empty() {
                format!("/{name}")
            } else {
                format!("{prefix}/{name}")
            };
            return Some((collection, singular(segment)));
        }
        let parent = format!("/{name}/:{}", singular(segment));
        prefix = if prefix.is_empty() {
            parent
        } else {
            format!("{prefix}{parent}")
        };
    }
    None
}

fn action_path(collection: &str, param: &str, action: &str) -> String {
    let member = format!("{collection}/:{param}");
    match action {
        "index" | "store" => collection.to_string(),
        "create" => format!("{collection}/create"),
        "show" | "update" | "destroy" => member,
        "edit" => format!("{member}/edit"),
        _ => collection.to_string(),
    }
}

fn singular(name: &str) -> String {
    name.strip_suffix('s')
        .filter(|stem| !stem.is_empty())
        .unwrap_or(name)
        .to_string()
}

pub fn static_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('$')
        && path
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '/' | '.'))
}
