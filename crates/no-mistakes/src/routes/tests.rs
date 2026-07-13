use super::*;
use std::fs;
use std::process::Command;

fn git_init(dir: &Path) {
    let output = Command::new("git")
        .args(["init", "-q", "--initial-branch=main"])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_add_all(dir: &Path) {
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(dir: &Path, path: &str, content: &str) {
    let full = dir.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, content).unwrap();
}

#[test]
fn non_git_route_discovery_applies_gitignore() {
    let dir = crate::test_support::materialize_gitignore_fixture("non-git-discovery");

    let routes = collect_routes(dir.path(), &["page"]);

    assert!(routes.iter().any(|route| route.pattern == "/app"));
    assert!(!routes.iter().any(|route| route.pattern == "/ignored"));
}

#[test]
fn test_path_to_route_pattern() {
    assert_eq!(path_to_route_pattern(Path::new("page.tsx")), "/");
    assert_eq!(path_to_route_pattern(Path::new("users/page.tsx")), "/users");
    assert_eq!(
        path_to_route_pattern(Path::new("(auth)/login/page.tsx")),
        "/login"
    );
    assert_eq!(
        path_to_route_pattern(Path::new("@sidebar/settings/page.tsx")),
        "/settings"
    );
    assert_eq!(
        path_to_route_pattern(Path::new("blog/[slug]/page.tsx")),
        "/blog/:slug"
    );
    assert_eq!(
        path_to_route_pattern(Path::new("shop/[[...rest]]/page.tsx")),
        "/shop/**"
    );
    assert_eq!(
        path_to_route_pattern(Path::new("docs/[...all]/page.tsx")),
        "/docs/*"
    );
    assert_eq!(
        path_to_route_pattern(Path::new("(group)/@parallel/page.tsx")),
        "/"
    );

    // Test non-normal components
    assert_eq!(path_to_route_pattern(Path::new("a/../b/page.tsx")), "/a/b");
}

#[test]
fn test_collect_routes() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    fs::create_dir(&app).unwrap();
    fs::write(app.join("page.tsx"), "").unwrap();
    fs::create_dir(app.join("users")).unwrap();
    fs::write(app.join("users/page.tsx"), "").unwrap();
    fs::write(app.join("not-a-page.ts"), "").unwrap();

    let routes = collect_routes(&app, &["page"]);
    assert_eq!(routes.len(), 2);
    assert_eq!(routes[0].pattern, "/");
    assert_eq!(routes[1].pattern, "/users");

    // Test sorting tiebreaker
    fs::write(app.join("users/layout.tsx"), "").unwrap();
    let routes = collect_routes(&app, &["page", "layout"]);
    assert_eq!(routes.len(), 3);
}

#[test]
fn test_collect_routes_missing_root() {
    let routes = collect_routes(Path::new("missing"), &["page"]);
    assert!(routes.is_empty());
}

#[test]
fn test_collect_routes_empty() {
    let dir = tempfile::tempdir().unwrap();
    let routes = collect_routes(dir.path(), &["page"]);
    assert!(routes.is_empty());
}

#[test]
fn dot_directories_are_excluded_from_route_scanning() {
    // Use the fixture root (which contains both app/ and .next/) as frontend_root.
    // Without the fix, the scanner would enter .next/ and pick up phantom routes.
    let fixture_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/nextjs-routes/skip-dot-next/fixture");
    let routes = collect_routes(&fixture_root, &["page", "route"]);
    let patterns: Vec<&str> = routes.iter().map(|r| r.pattern.as_str()).collect();
    assert!(
        !patterns.iter().any(|p| p.contains(".next")),
        "should not find routes inside .next/, got: {patterns:?}"
    );
    assert!(
        patterns.iter().any(|p| p.contains("about")),
        "should find /app/about route, got: {patterns:?}"
    );
}

/// Regression test for the route-collection walk visiting large gitignored
/// directories (e.g. a dependency store) instead of deriving candidates from the
/// git-visible file list. Before the fix, `collect_routes` always did a raw
/// recursive `WalkDir` walk with no `.gitignore` awareness beyond skipping
/// dot-directories, so a `page.tsx` file anywhere under a large ignored directory
/// (e.g. `node_modules`) would still be visited and returned as a route, even
/// though none of its contents are ever git-visible and thus can never be part of
/// a real frontend build.
#[test]
fn collect_routes_does_not_walk_gitignored_directory() {
    let dir = tempfile::tempdir().unwrap();
    git_init(dir.path());
    write(dir.path(), ".gitignore", "dependency-store/\n");
    write(dir.path(), "app/page.tsx", "");
    write(dir.path(), "dependency-store/nested/app/trap/page.tsx", "");
    git_add_all(dir.path());

    let routes = collect_routes(dir.path(), &["page"]);

    assert!(routes
        .iter()
        .any(|r| r.file == dir.path().join("app/page.tsx")));
    assert!(!routes
        .iter()
        .any(|r| r.file.starts_with(dir.path().join("dependency-store"))));
}

/// A file can be git-tracked (staged/committed) yet missing on disk — e.g. deleted
/// with `rm` rather than `git rm`. The git-derived path must not surface such a
/// route, matching `collect_routes_by_walk`'s implicit guarantee that every entry
/// it finds actually exists.
#[test]
fn collect_routes_excludes_git_tracked_file_missing_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    git_init(dir.path());
    write(dir.path(), "app/page.tsx", "");
    write(dir.path(), "app/deleted/page.tsx", "");
    git_add_all(dir.path());
    std::fs::remove_file(dir.path().join("app/deleted/page.tsx")).unwrap();

    let routes = collect_routes(dir.path(), &["page"]);

    assert!(routes
        .iter()
        .any(|r| r.file == dir.path().join("app/page.tsx")));
    assert!(!routes
        .iter()
        .any(|r| r.file == dir.path().join("app/deleted/page.tsx")));
}

/// `git ls-files` can surface tracked files under dot-directories (e.g. a
/// committed `.next/` build-output fixture, mirroring the
/// `test-cases/nextjs-routes/skip-dot-next` fixture above, which is itself
/// committed to this repo) that the raw walk's dot-directory `filter_entry` would
/// never have descended into. The git-derived path must apply the same
/// dot-directory skip, or tracked build output would appear as phantom routes
/// only when git is available — the opposite of what a git-repo-aware fast path
/// should do.
#[test]
fn collect_routes_excludes_dot_directories_from_git_visible_files() {
    let dir = tempfile::tempdir().unwrap();
    git_init(dir.path());
    write(dir.path(), "app/page.tsx", "");
    write(dir.path(), ".next/server/app/page.tsx", "");
    git_add_all(dir.path());

    let routes = collect_routes(dir.path(), &["page"]);

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].file, dir.path().join("app/page.tsx"));
}

/// Outside a Git repository, route-specific dot-directory filtering still
/// applies to the shared ignore-aware candidate list.
#[test]
fn collect_routes_applies_dot_dir_filter_outside_git() {
    let dir = tempfile::tempdir().unwrap();
    let frontend_root = dir.path().join("app");
    write(&frontend_root, "page.tsx", "");
    write(&frontend_root, ".hidden/page.tsx", "");

    let routes = collect_routes(&frontend_root, &["page"]);

    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].file, frontend_root.join("page.tsx"));
}
