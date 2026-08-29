use crate::playwright::fsutil::{absolutize, relative_string};
use crate::playwright::matcher;
use crate::playwright::url::{is_dynamic_pattern_segment, is_ignored, normalize_url};
use std::path::Path;

#[test]
fn normalize_url_handles_relative_absolute_base_and_external() {
    let bases = vec!["http://localhost:3000/".to_string()];
    assert_eq!(
        normalize_url("/users/42", &bases),
        Some("/users/42".to_string())
    );
    assert_eq!(
        normalize_url("http://localhost:3000/users/42", &bases),
        Some("/users/42".to_string())
    );
    assert_eq!(
        normalize_url("http://localhost:3000", &bases),
        Some("/".to_string())
    );
    assert_eq!(normalize_url("http://localhost:3000x", &bases), None);
    assert_eq!(normalize_url("https://example.com/users/42", &bases), None);
}

#[test]
fn ignore_routes_match_exact_and_dynamic_patterns() {
    assert!(is_ignored("/settings", &["/settings".to_string()]));
    assert!(is_ignored("/users/42", &["/users/:id".to_string()]));
    assert!(!is_ignored("/admin", &["/settings".to_string()]));
}

#[test]
fn normalize_url_handles_edge_cases() {
    let base_urls = vec!["http://localhost:3000".to_string()];
    assert_eq!(normalize_url("//google.com", &base_urls), None);
    assert_eq!(
        normalize_url("http://localhost:3000", &base_urls),
        Some("/".to_string())
    );
    assert_eq!(
        normalize_url("http://localhost:3000/", &base_urls),
        Some("/".to_string())
    );
    assert_eq!(normalize_url("http://other.com", &base_urls), None);
}

#[test]
fn path_helpers_handle_absolute_and_relative_paths() {
    let cwd = std::env::current_dir().unwrap();
    let absolute = cwd.join("tmp");
    assert_eq!(absolutize(&absolute).unwrap(), absolute);
    assert_eq!(absolutize(Path::new(".")).unwrap(), cwd.join("."));
    match cwd.parent() {
        Some(parent) => {
            let outside = parent.join("other/file.ts");
            assert_eq!(
                relative_string(&cwd, &outside),
                outside.to_string_lossy().replace('\\', "/")
            );
        }
        None => {
            let rendered = relative_string(&cwd, unrelated_outside_path());
            assert!(!rendered.is_empty());
            assert!(!rendered.contains('\\'));
        }
    }
}

#[test]
fn relative_string_handles_filesystem_root_without_a_parent() {
    let root = filesystem_root();
    assert!(root.parent().is_none());
    let outside = unrelated_outside_path();
    let rendered = relative_string(root, outside);
    assert!(!rendered.contains('\\'));
    if cfg!(windows) {
        assert_eq!(rendered, outside.to_string_lossy().replace('\\', "/"));
    } else {
        // Every absolute Unix path is under `/`, so the root prefix is stripped.
        assert_eq!(rendered, "other/file.ts");
    }
}

fn filesystem_root() -> &'static Path {
    if cfg!(windows) {
        Path::new(r"C:\")
    } else {
        Path::new("/")
    }
}

fn unrelated_outside_path() -> &'static Path {
    if cfg!(windows) {
        Path::new(r"Z:\other\file.ts")
    } else {
        Path::new("/other/file.ts")
    }
}

#[test]
fn compiled_route_matching_handles_edge_segments() {
    assert!(is_dynamic_pattern_segment(":id"));
    assert!(is_dynamic_pattern_segment("*"));
    assert!(is_dynamic_pattern_segment("**"));
    assert!(!is_dynamic_pattern_segment("users"));

    assert_eq!(
        matcher::reference_segments("/users/42/?tab=profile"),
        vec!["users", "42"]
    );
    assert_eq!(
        matcher::pattern_segments("/users/:id"),
        vec!["users", ":id"]
    );
    assert!(matcher::matches_segments(
        &["shop"],
        &["shop".to_string(), "**".to_string()]
    ));
    assert!(!matcher::matches_segments(
        &["shop"],
        &["shop".to_string(), "item".to_string()]
    ));
}
