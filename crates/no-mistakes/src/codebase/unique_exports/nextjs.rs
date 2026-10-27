use std::path::Path;

#[path = "nextjs_names.rs"]
mod names;

pub(super) fn is_framework_export(rel: &str, name: &str, is_nextjs_project: bool) -> bool {
    if !is_nextjs_project {
        return false;
    }
    let rel = normalized_rel(rel);
    let file_name = file_name(&rel);
    let stem = convention_stem(file_name);
    match router_kind(&rel) {
        Some(RouterKind::App) => names::app_unique_export(stem, name),
        Some(RouterKind::Pages) => names::is_pages_router_export(name),
        None => false,
    }
}

/// Framework-owned file relative to a Next.js project root.
///
/// `path` counts when it sits under `project_root` and matches an App Router
/// or Pages Router file, root or `src/` middleware, instrumentation, proxy, or
/// instrumentation-client, or a root `next.config` code file. Callers decide
/// which roots are `type: nextjs`.
pub(crate) fn is_framework_entrypoint_file(path: &Path, project_root: &Path) -> bool {
    let Some(rel) = project_rel(path, project_root) else {
        return false;
    };
    is_entrypoint_rel(&rel)
}

/// True when Next.js consumes this export.
///
/// `next.config`, `middleware`, and `instrumentation` are whole-file because
/// the framework loads the module. Every other convention exempts only the
/// names Next reads.
pub(crate) fn framework_consumed_export(path: &Path, project_root: &Path, name: &str) -> bool {
    let Some(rel) = project_rel(path, project_root) else {
        return false;
    };
    consumed_rel(&rel, name)
}

fn project_rel(path: &Path, project_root: &Path) -> Option<String> {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let project_root = crate::codebase::ts_resolver::normalize_path(project_root);
    path.starts_with(&project_root)
        .then(|| crate::codebase::ts_source::relative_slash_path(&project_root, &path))
}

fn is_entrypoint_rel(rel: &str) -> bool {
    let rel = normalized_rel(rel);
    let (parent, file_name) = split_rel(&rel);
    if is_next_config(file_name) {
        return parent.is_empty();
    }
    let Some(stem) = code_stem(file_name) else {
        return false;
    };
    if is_root_module(parent, stem) || is_pages_route(parent, stem) {
        return true;
    }
    if matches!(stem, "proxy" | "instrumentation-client") {
        return parent.is_empty() || parent == "src";
    }
    names::is_app_entry_stem(stem) && under_app(parent)
}

fn consumed_rel(rel: &str, name: &str) -> bool {
    let rel = normalized_rel(rel);
    let (parent, file_name) = split_rel(&rel);
    if is_next_config(file_name) {
        return parent.is_empty();
    }
    let Some(stem) = code_stem(file_name) else {
        return false;
    };
    if is_root_module(parent, stem) {
        return true;
    }
    if is_pages_route(parent, stem) {
        return name == "default" || names::is_pages_router_export(name);
    }
    if stem == "proxy" && (parent.is_empty() || parent == "src") {
        return matches!(name, "default" | "proxy" | "config");
    }
    if stem == "instrumentation-client" && (parent.is_empty() || parent == "src") {
        return name == "onRouterTransitionStart";
    }
    names::is_app_entry_stem(stem) && under_app(parent) && names::app_consumed_export(stem, name)
}

fn is_root_module(parent: &str, stem: &str) -> bool {
    matches!(stem, "middleware" | "instrumentation") && (parent.is_empty() || parent == "src")
}

fn is_pages_route(parent: &str, stem: &str) -> bool {
    let under = parent == "pages"
        || parent.starts_with("pages/")
        || parent == "src/pages"
        || parent.starts_with("src/pages/");
    if !under {
        return false;
    }
    match stem.strip_prefix('_') {
        Some("app" | "document" | "error") => true,
        Some(_) => false,
        None => true,
    }
}

fn under_app(parent: &str) -> bool {
    parent == "app"
        || parent.starts_with("app/")
        || parent == "src/app"
        || parent.starts_with("src/app/")
}

fn is_next_config(file_name: &str) -> bool {
    let Some(extension) = file_name.strip_prefix("next.config.") else {
        return false;
    };
    matches!(extension, "js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
}

const CODE_EXTENSIONS: &[&str] = &[".tsx", ".ts", ".jsx", ".js", ".mts", ".cts", ".mjs", ".cjs"];

fn code_stem(file_name: &str) -> Option<&str> {
    CODE_EXTENSIONS
        .iter()
        .find_map(|extension| file_name.strip_suffix(extension))
}

fn convention_stem(file_name: &str) -> &str {
    code_stem(file_name).unwrap_or(file_name)
}

fn normalized_rel(rel: &str) -> String {
    let rel = rel.replace('\\', "/");
    let rel = rel.trim_start_matches("./");
    rel.trim_start_matches('/').to_owned()
}

fn split_rel(rel: &str) -> (&str, &str) {
    match rel.rsplit_once('/') {
        Some((parent, file_name)) => (parent, file_name),
        None => ("", rel),
    }
}

fn file_name(rel: &str) -> &str {
    split_rel(rel).1
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RouterKind {
    App,
    Pages,
}

fn router_kind(rel: &str) -> Option<RouterKind> {
    rel.split('/').find_map(|segment| match segment {
        "app" => Some(RouterKind::App),
        "pages" => Some(RouterKind::Pages),
        _ => None,
    })
}
