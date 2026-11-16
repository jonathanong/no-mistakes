use super::{graph, home, resolution, resolve, Facts};
use crate::codebase::ts_resolver::normalize_path;
use crate::fx::PathSet;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn assert_cached_repeat(
    located: &home::Locate<'_>,
    from: &Path,
    hit: &Path,
    observer: &crate::diagnostics::InvocationObserver,
) {
    assert_eq!(
        resolve::resolve(located, from, "../Cargo.toml"),
        Some(hit.to_path_buf())
    );
    let resolved = observer.snapshot().work["resolver.computations"];
    assert!(resolved > 0);
    assert_eq!(
        resolve::resolve(located, from, "../Cargo.toml"),
        Some(hit.to_path_buf())
    );
    assert_eq!(observer.snapshot().work["resolver.computations"], resolved);
    assert_eq!(
        resolve::resolve(located, from, "./no-such-registry-target.mts"),
        None
    );
    let missed = observer.snapshot().work["resolver.computations"];
    assert!(missed > resolved);
    assert_eq!(
        resolve::resolve(located, from, "./no-such-registry-target.mts"),
        None
    );
    assert_eq!(observer.snapshot().work["resolver.computations"], missed);
}

#[test]
fn repeated_dispatches_reuse_a_resolver_built_without_a_session() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let from = root.join("src/lib.rs");
    let cargo = normalize_path(&root.join("Cargo.toml"));
    let config = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..crate::codebase::ts_resolver::TsConfig::default()
    };
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, config, None);
    let visible = PathSet::from_iter([from.clone(), cargo.clone()]);
    let linked = graph(&[(&from, &cargo)]);
    let facts = Facts {
        files: HashMap::new(),
    };
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let catalog_resolution = super::super::super::Resolution {
            catalog: Some(&catalog),
            workspace: None,
            session: None,
        };
        let located = home::Locate {
            graph: &linked,
            facts: &facts,
            resolution: &catalog_resolution,
            visible: &visible,
            resolver: resolve::RegistryResolver::prepare(&catalog_resolution, &visible),
        };
        assert_cached_repeat(&located, &from, &cargo, &observer);

        let bare = resolution();
        let located = home::Locate {
            graph: &linked,
            facts: &facts,
            resolution: &bare,
            visible: &visible,
            resolver: resolve::RegistryResolver::prepare(&bare, &visible),
        };
        assert_cached_repeat(&located, &from, &cargo, &observer);
    });
}
