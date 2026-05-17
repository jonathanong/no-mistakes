use super::{init_rayon_threads, rayon_thread_count, resolve_optional_root, resolve_root, JobsArg};
use std::path::Path;

#[test]
fn resolve_root_preserves_absolute_paths() {
    let cwd = Path::new("/repo");
    let root = Path::new("/workspace/app");

    assert_eq!(resolve_root(root, cwd), root);
}

#[test]
fn resolve_root_joins_relative_paths() {
    assert_eq!(
        resolve_root(Path::new("app"), Path::new("/repo")),
        Path::new("/repo/app")
    );
}

#[test]
fn resolve_optional_root_defaults_to_cwd() {
    let cwd = Path::new("/repo");

    assert_eq!(resolve_optional_root(None, cwd), cwd);
}

#[test]
fn resolve_optional_root_resolves_provided_root() {
    assert_eq!(
        resolve_optional_root(Some(Path::new("app")), Path::new("/repo")),
        Path::new("/repo/app")
    );
}

#[test]
fn init_rayon_threads_uses_cpu_default_without_jobs_or_env() {
    init_rayon_threads(JobsArg { jobs: 0 });
}

#[test]
fn rayon_thread_count_prefers_jobs_then_env_then_cpu_default() {
    assert_eq!(rayon_thread_count(JobsArg { jobs: 4 }, Some("2")), 4);
    assert_eq!(rayon_thread_count(JobsArg { jobs: 0 }, Some("2")), 2);
    assert_eq!(
        rayon_thread_count(JobsArg { jobs: 0 }, Some("not-a-number")),
        num_cpus::get()
    );
    assert_eq!(
        rayon_thread_count(JobsArg { jobs: 0 }, None),
        num_cpus::get()
    );
}
