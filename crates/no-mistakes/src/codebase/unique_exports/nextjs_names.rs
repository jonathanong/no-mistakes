pub(super) fn app_unique_export(stem: &str, name: &str) -> bool {
    match stem {
        "page" | "layout" => is_app_page_or_layout_export(name),
        "opengraph-image" | "twitter-image" | "icon" | "apple-icon" => {
            is_app_metadata_asset_export(name)
        }
        "route" => is_app_route_export(name),
        _ => false,
    }
}

/// Names Next.js reads from an App Router convention file, including `default`.
pub(super) fn app_consumed_export(stem: &str, name: &str) -> bool {
    match stem {
        "page" | "layout" => name == "default" || is_app_page_or_layout_export(name),
        "route" => is_app_route_export(name),
        "opengraph-image" | "twitter-image" | "icon" | "apple-icon" => {
            name == "default" || is_app_metadata_asset_export(name)
        }
        _ => name == "default" || is_route_segment_config_export(name),
    }
}

pub(super) fn is_app_entry_stem(stem: &str) -> bool {
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

pub(super) fn is_pages_router_export(name: &str) -> bool {
    matches!(
        name,
        "getStaticProps"
            | "getStaticPaths"
            | "getServerSideProps"
            | "getInitialProps"
            | "config"
            | "reportWebVitals"
    )
}
