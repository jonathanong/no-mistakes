use super::*;

#[test]
fn source_worker_defaults_and_environment_are_explicit() {
    assert_eq!(worker_count(Some("4")), 4);
    for value in [None, Some("0"), Some("invalid"), Some("-1")] {
        assert_eq!(worker_count(value), num_cpus::get());
    }
    assert_eq!(
        pool().install(rayon::current_num_threads),
        worker_count(std::env::var("RAYON_NUM_THREADS").ok().as_deref())
    );
}
