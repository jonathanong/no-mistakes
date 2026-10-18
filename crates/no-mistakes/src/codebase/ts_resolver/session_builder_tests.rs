use super::*;

fn preserves_original_scope(builder: u8) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/tsconfig-paths/fixture")
        .canonicalize()
        .unwrap();
    let importer = root.join("enqueue.ts");
    let target = root.join("queues/email.ts");
    let full = crate::fx::PathSet::from_iter([importer.clone(), target.clone()]);
    let narrow = crate::fx::PathSet::from_iter([importer.clone()]);
    let config = TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    let original = ImportResolver::new_in_session(&config, Some(&full), &session);
    let specifier = if builder == 2 {
        "queues/email"
    } else {
        "./queues/email"
    };
    let original_target = (builder != 2).then(|| target.clone());
    assert_eq!(original.resolve(specifier, &importer), original_target);
    let changed = match builder {
        0 => ImportResolver::new_in_session(&config, Some(&full), &session).with_visible(&narrow),
        1 => ImportResolver::new_owned_in_session(
            Arc::new(config.clone()),
            Arc::new(full.clone()),
            &session,
        )
        .with_owned_visible(Arc::new(narrow)),
        _ => ImportResolver::new_in_session(&config, Some(&full), &session)
            .with_queue_compatibility(&root),
    };
    assert_eq!(
        changed.resolve(specifier, &importer),
        (builder == 2).then_some(target)
    );
    // Changing one facade must neither erase nor poison another facade's table.
    assert_eq!(original.resolve(specifier, &importer), original_target);
    assert_eq!(observer.snapshot().work["resolver.computations"], 2);
}

#[test]
fn session_builder_borrowed_visibility_cannot_poison_the_original_scope() {
    preserves_original_scope(0);
}

#[test]
fn session_builder_owned_visibility_cannot_poison_the_original_scope() {
    preserves_original_scope(1);
}

#[test]
fn session_builder_queue_policy_cannot_poison_the_original_scope() {
    preserves_original_scope(2);
}
