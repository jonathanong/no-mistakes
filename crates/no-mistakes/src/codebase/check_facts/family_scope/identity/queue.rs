use super::{
    applies, configure_family_path_scopes, fixture, queue_plan, queue_report, FamilyPathScope,
};
use crate::codebase::check_facts::collect_check_facts;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;

#[test]
fn queue_memmem_scope_preserves_factory_reexport_facts() {
    let root = fixture("queue-ast-hop/factory-reexports/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    assert_eq!(
        scoped.queue_paths,
        FamilyPathScope::All,
        "enq/unresolved.mts imports @external/queues, which the relative closure cannot follow"
    );
    assert_eq!(
        queue_report(&root, collect_check_facts(&root, files.clone(), scoped)),
        queue_report(&root, collect_check_facts(&root, files, queue_plan(&root))),
    );
}

#[test]
fn queue_generic_add_importer_stays_a_memmem_seed() {
    let root = fixture("check-facts/family-scope-queue-generic");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let enqueue = normalize_path(&root.join("src/enqueue.ts"));
    assert!(
        applies(&scoped.queue_paths, &enqueue),
        "generic queue.add<T>(...) must seed the importer so its producer is not dropped"
    );
    let scoped_report = queue_report(&root, collect_check_facts(&root, files.clone(), scoped));
    let full_report = queue_report(&root, collect_check_facts(&root, files, queue_plan(&root)));
    assert!(
        !scoped_report.contains("unmatched-worker"),
        "generic add must keep the sendWelcome producer: {scoped_report}"
    );
    assert_eq!(scoped_report, full_report);
}

#[test]
fn queue_split_dot_add_importer_stays_a_memmem_seed() {
    let root = fixture("check-facts/family-scope-queue-split-dot");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let enqueue = normalize_path(&root.join("src/enqueue.ts"));
    let enqueue_source =
        std::fs::read_to_string(root.join("src/enqueue.ts")).expect("enqueue fixture");
    assert!(
        enqueue_source
            .lines()
            .filter(|line| line.trim_start().starts_with("//"))
            .all(|line| !line.contains("add")),
        "comments must not contain the old needle, or this test would pass without the split-dot seed"
    );
    assert!(
        applies(&scoped.queue_paths, &enqueue),
        "q.\\n add / q. add / q./*c*/add must seed the importer so those producers are not dropped"
    );
    let scoped_report = queue_report(&root, collect_check_facts(&root, files.clone(), scoped));
    let full_report = queue_report(&root, collect_check_facts(&root, files, queue_plan(&root)));
    assert!(
        !scoped_report.contains("unmatched-worker"),
        "split-dot add must keep the sendWelcome producer: {scoped_report}"
    );
    assert_eq!(scoped_report, full_report);
}

#[test]
fn queue_wait_routes_stay_in_memmem_scope() {
    let root = fixture("queue-ast-hop/factory-roles/fixture");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut scoped = queue_plan(&root);
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
        .expect("queue fixture config");
    configure_family_path_scopes(&root, &files, &config, sources.as_ref(), &mut scoped).unwrap();
    let wait = normalize_path(&root.join("routes/wait.ts"));
    assert!(
        applies(&scoped.queue_paths, &wait),
        "addAndWait routes must stay in the queue extraction scope"
    );
    assert_eq!(
        queue_report(&root, collect_check_facts(&root, files.clone(), scoped)),
        queue_report(&root, collect_check_facts(&root, files, queue_plan(&root))),
    );
}
