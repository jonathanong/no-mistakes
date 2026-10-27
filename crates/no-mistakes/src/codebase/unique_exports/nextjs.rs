use std::path::Path;

pub(super) fn is_framework_export(rel: &str, name: &str, is_nextjs_project: bool) -> bool {
    if !is_nextjs_project {
        return false;
    }
    let rel = rel.replace('\\', "/");
    let file_name = rel.rsplit('/').next().unwrap_or("");
    let stem = convention_stem(file_name);
    match router_kind(&rel) {
        Some(RouterKind::App) => match stem {
            "page" | "layout" => is_app_page_or_layout_export(name),
            "opengraph-image" | "twitter-image" | "icon" | "apple-icon" => {
                is_app_metadata_asset_export(name)
            }
            "route" => is_app_route_export(name),
            _ => false,
        },
        Some(RouterKind::Pages) => is_pages_router_export(name),
        None => false,
    }
}

/// Framework-owned file relative to a Next.js project root.
///
/// `path` counts only when it sits under `project_root` and matches the App
/// Router file convention, root or `src/` middleware/instrumentation, or a
/// root `next.config` code file. Callers decide which roots are `type: nextjs`.
pub(crate) fn is_framework_entrypoint_file(path: &Path, project_root: &Path) -> bool {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let project_root = crate::codebase::ts_resolver::normalize_path(project_root);
    if !path.starts_with(&project_root) {
        return false;
    }
    let rel = crate::codebase::ts_source::relative_slash_path(&project_root, &path);
    is_entrypoint_rel(&rel)
}

fn is_entrypoint_rel(rel: &str) -> bool {
    let rel = rel.replace('\\', "/");
    let rel = rel.trim_start_matches("./");
    let rel = rel.trim_start_matches('/');
    let (parent, file_name) = match rel.rsplit_once('/') {
        Some((parent, file_name)) => (parent, file_name),
        None => ("", rel),
    };
    if is_next_config(file_name) {
        return parent.is_empty();
    }
    let Some(stem) = code_stem(file_name) else {
        return false;
    };
    if matches!(stem, "middleware" | "instrumentation") {
        return parent.is_empty() || parent == "src";
    }
    is_app_entry_stem(stem) && under_app(parent)
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

fn is_app_entry_stem(stem: &str) -> bool {
    matches!(
        stem,
        "page"
            | "layout"
            | "loading"
            | "error"
            | "global-error"
            | "not-found"
            | "template"
            | "default"
            | "route"
            | "manifest"
            | "robots"
            | "sitemap"
            | "icon"
            | "apple-icon"
            | "opengraph-image"
            | "twitter-image"
    )
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

fn is_app_page_or_layout_export(name: &str) -> bool {
    matches!(
        name,
        "metadata" | "generateMetadata" | "viewport" | "generateViewport" | "generateStaticParams"
    ) || is_route_segment_config_export(name)
}

fn is_app_route_export(name: &str) -> bool {
    matches!(
        name,
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS"
    ) || is_route_segment_config_export(name)
}

fn is_app_metadata_asset_export(name: &str) -> bool {
    matches!(name, "alt" | "size" | "contentType") || is_route_segment_config_export(name)
}

fn is_route_segment_config_export(name: &str) -> bool {
    matches!(
        name,
        "dynamic"
            | "dynamicParams"
            | "revalidate"
            | "fetchCache"
            | "runtime"
            | "preferredRegion"
            | "maxDuration"
            | "experimental_ppr"
    )
}

fn is_pages_router_export(name: &str) -> bool {
    matches!(
        name,
        "getStaticProps" | "getStaticPaths" | "getServerSideProps" | "config" | "reportWebVitals"
    )
}
