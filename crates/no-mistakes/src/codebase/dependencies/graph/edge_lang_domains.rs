fn emit_route_edges(
    root: &Path,
    facts: &LangFactMap,
    options: &GraphConfigOptions,
    edges: &mut Vec<Edge>,
    interner: &PathInterner,
) {
    for file in facts.files.values() {
        if !route_file_allowed(root, &file.path, options) {
            continue;
        }
        for (_, handler) in &file.route_handlers {
            if let Some(targets) = facts.files_by_module.get(handler) {
                let scoped: std::collections::BTreeSet<_> = targets
                    .iter()
                    .filter(|target| {
                        facts
                            .files
                            .get(*target)
                            .is_some_and(|other| same_lang_package(file, other))
                    })
                    .cloned()
                    .collect();
                push_file_edges(edges, &file.path, &scoped, EdgeKind::RouteRef, interner);
            }
            for name in route_handler_names(handler)
                .into_iter()
                .flat_map(|name| aliased_route_names(file, name))
            {
                if let Some(targets) = facts.declarations.get(&name) {
                    let scoped: std::collections::BTreeSet<_> = targets
                        .iter()
                        .filter(|target| {
                            facts.files.get(*target).is_some_and(|other| {
                                same_lang_package(file, other)
                                    && handler_module_matches(handler, other)
                            })
                        })
                        .cloned()
                        .collect();
                    push_file_edges(edges, &file.path, &scoped, EdgeKind::RouteRef, interner);
                }
            }
        }
    }
}

fn route_file_allowed(root: &Path, path: &Path, options: &GraphConfigOptions) -> bool {
    let Some(globset) = options.project_route_globset.as_ref() else {
        return true;
    };
    let rel = path.strip_prefix(root).unwrap_or(path);
    globset.is_match(rel.to_string_lossy().as_ref())
}

fn same_lang_package(file: &LangFileFacts, other: &LangFileFacts) -> bool {
    file.package.is_none() || file.package == other.package
}

fn handler_module_matches(handler: &str, file: &LangFileFacts) -> bool {
    let view = remap_aliased_handler(file, &normalize_route_handler(handler));
    if !view.contains('.') || view.contains("::") || view.contains('#') || view.contains('/') {
        return true;
    }
    let Some(module) = file.module.as_deref() else {
        return true;
    };
    if module == view || view.starts_with(&format!("{module}.")) {
        return true;
    }
    view.rsplit_once('.')
        .is_some_and(|(parent, _)| module == parent || module.ends_with(&format!(".{parent}")))
}

fn aliased_route_names(file: &LangFileFacts, name: String) -> Vec<String> {
    let mut names = vec![name.clone()];
    for import in &file.imports {
        let Some((alias, target)) = import.split_once('=') else {
            continue;
        };
        if alias == name {
            names.push(target.to_string());
            if let Some(short) = target.rsplit('.').next() {
                names.push(short.to_string());
            }
        }
    }
    names
}

fn remap_aliased_handler(file: &LangFileFacts, view: &str) -> String {
    let Some((prefix, rest)) = view.split_once('.') else {
        return view.to_string();
    };
    file.imports
        .iter()
        .find_map(|import| {
            let (alias, target) = import.split_once('=')?;
            (alias == prefix).then(|| format!("{target}.{rest}"))
        })
        .unwrap_or_else(|| view.to_string())
}

fn normalize_route_handler(handler: &str) -> String {
    let trimmed = handler.replace(['\'', '"', ' '], "");
    trimmed
        .strip_suffix("()")
        .unwrap_or(trimmed.as_str())
        .strip_suffix(".as_view")
        .unwrap_or(trimmed.as_str())
        .to_string()
}

fn reference_target_allowed(file: &LangFileFacts, target: &LangFileFacts, reference: &str) -> bool {
    if file.module.is_some() && file.module == target.module {
        return true;
    }
    if reference.contains("::") && same_lang_package(file, target) {
        return true;
    }
    if target.module.as_deref().is_some_and(|module| {
        file.imports
            .iter()
            .any(|import| import_reaches_module(import, module, reference))
    }) {
        return true;
    }
    if file.module.is_some() {
        return false;
    }
    same_lang_package(file, target)
}

fn import_reaches_module(import: &str, module: &str, reference: &str) -> bool {
    import == module
        || import == reference
        || import == format!("{module}.{reference}")
        || import == format!("{module}/{reference}")
        || import.starts_with(&format!("{module}."))
        || Path::new(import).file_stem().and_then(|name| name.to_str())
            == Path::new(module).file_stem().and_then(|name| name.to_str())
}

fn route_handler_names(handler: &str) -> Vec<String> {
    let trimmed = handler.replace(['\'', '"', ' '], "");
    if let Some((controller, _)) = trimmed.split_once('#') {
        return rails_controller_names(controller);
    }
    if let Some((class, _)) = trimmed.split_once("::") {
        let short = class.rsplit('\\').next().unwrap_or(class).to_string();
        return vec![short, class.to_string()];
    }
    let view = normalize_route_handler(&trimmed);
    let mut names = vec![view.clone()];
    if let Some((parent, last)) = view.rsplit_once('.') {
        names.push(last.to_string());
        // Phoenix `Controller.action` must also resolve the controller module.
        if last.starts_with(|ch: char| ch.is_ascii_lowercase()) {
            names.push(parent.to_string());
            if let Some((_, type_name)) = parent.rsplit_once('.') {
                names.push(type_name.to_string());
            }
        }
    }
    names
}

fn rails_controller_names(controller: &str) -> Vec<String> {
    let parts: Vec<&str> = controller
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let last = parts.last().copied().unwrap_or(controller);
    let mut class = last.to_string();
    if !class.ends_with("Controller") {
        class.push_str("Controller");
    }
    let class = snake_to_pascal(&class);
    if parts.len() <= 1 {
        return vec![class];
    }
    let namespace = parts[..parts.len() - 1]
        .iter()
        .map(|part| snake_to_pascal(part))
        .collect::<Vec<_>>()
        .join("::");
    vec![format!("{namespace}::{class}")]
}

fn snake_to_pascal(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let first = chars.next().expect("non-empty");
            first.to_ascii_uppercase().to_string() + chars.as_str()
        })
        .collect()
}
