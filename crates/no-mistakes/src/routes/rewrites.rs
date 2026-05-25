use super::Route;
use crate::config::v2::schema::RewriteRule;
use crate::playwright::matcher;
use std::path::PathBuf;

pub fn normalize_nextjs_pattern(pattern: &str) -> String {
    let raw = pattern
        .strip_prefix('/')
        .unwrap_or(pattern)
        .trim_end_matches('/');
    if raw.is_empty() {
        return "/".to_string();
    }
    let segments: Vec<&str> = raw.split('/').filter(|s| !s.is_empty()).collect();
    let last = segments.len().saturating_sub(1);
    let normalized: Vec<String> = segments
        .iter()
        .enumerate()
        .map(|(i, seg)| {
            if let Some(stripped) = seg.strip_prefix(':') {
                if stripped.ends_with('*') && i == last {
                    "**".to_string()
                } else if stripped.ends_with('+') && i == last {
                    "*".to_string()
                } else {
                    seg.to_string()
                }
            } else {
                seg.to_string()
            }
        })
        .collect();
    format!("/{}", normalized.join("/"))
}

pub fn destination_contained_by_route(dest: &[&str], route: &[&str]) -> bool {
    let mut di = 0;
    let mut ri = 0;

    while ri < route.len() {
        let r = route[ri];
        let is_last_route = ri + 1 == route.len();

        if r == "**" && is_last_route {
            return true;
        }
        if r == "*" && is_last_route {
            if di >= dest.len() {
                return false;
            }
            return dest[di..] != ["**"];
        }

        let Some(&d) = dest.get(di) else {
            return false;
        };

        let d_is_wildcard = d == "**" || d == "*";
        let d_is_param = d.starts_with(':');
        let r_is_param = r.starts_with(':');

        if d_is_wildcard {
            let is_last_dest = di + 1 == dest.len();
            if !is_last_dest {
                return false;
            }
            if d == "**" {
                return r == "**" && is_last_route;
            }
            return (r == "*" || r == "**") && is_last_route;
        }

        if d_is_param {
            if !r_is_param && r != "*" && r != "**" {
                return false;
            }
        } else if !r_is_param && d != r {
            return false;
        }

        di += 1;
        ri += 1;
    }

    di == dest.len()
}

pub fn expand_rewrites(rewrites: &[RewriteRule], real_routes: &[Route]) -> Vec<Route> {
    let mut virtual_routes = Vec::new();
    let route_segments: Vec<Vec<String>> = real_routes
        .iter()
        .map(|r| {
            matcher::pattern_segments(&r.pattern)
                .into_iter()
                .map(str::to_string)
                .collect()
        })
        .collect();

    for rewrite in rewrites {
        let norm_source = normalize_nextjs_pattern(&rewrite.source);
        let norm_dest = normalize_nextjs_pattern(&rewrite.destination);
        if norm_source == norm_dest {
            continue;
        }
        let dest_segs = matcher::pattern_segments(&norm_dest);
        for (i, real) in real_routes.iter().enumerate() {
            let route_segs: Vec<&str> = route_segments[i].iter().map(String::as_str).collect();
            if destination_contained_by_route(&dest_segs, &route_segs) {
                virtual_routes.push(Route {
                    file: real.file.clone(),
                    pattern: norm_source.clone(),
                });
            }
        }
    }

    virtual_routes.sort_by(|a, b| a.pattern.cmp(&b.pattern).then_with(|| a.file.cmp(&b.file)));
    virtual_routes.dedup_by(|a, b| a.pattern == b.pattern && a.file == b.file);
    virtual_routes
}

pub fn expand_rewrites_as_tuples(
    rewrites: &[RewriteRule],
    real_routes: &[(PathBuf, String)],
) -> Vec<(PathBuf, String)> {
    let routes: Vec<Route> = real_routes
        .iter()
        .map(|(file, pattern)| Route {
            file: file.clone(),
            pattern: pattern.clone(),
        })
        .collect();
    expand_rewrites(rewrites, &routes)
        .into_iter()
        .map(|r| (r.file, r.pattern))
        .collect()
}

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;
