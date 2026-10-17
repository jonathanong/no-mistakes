use super::*;
use crate::codebase::postgres::query_annotation::{self, modules::Module};
use crate::codebase::postgres::EmbeddedSqlOptions;
use std::sync::Arc;

#[test]
fn resolverless_template_keeps_the_caller_type_without_a_resolver_capability() {
    fn check<F: Fn(&str, &Path) -> Option<PathBuf>>(resolve: &F) {
        let files = fx_map();
        let normal = super::super::new(&files, resolve, false);
        let isolated = super::super::new::without_resolver::<&F>(&files);
        assert!(normal.resolve.is_some());
        assert!(isolated.resolve.is_none());
        // The production caller supplies a resolver reference, whose Option
        // uses the null niche instead of enlarging the evaluator field.
        assert_eq!(
            std::mem::size_of_val(&normal.resolve),
            std::mem::size_of::<&F>()
        );
    }
    let resolve = |_: &str, _: &Path| None;
    check(&resolve);
}

#[test]
fn resolverless_evaluator_rejects_uncached_and_cached_import_targets() {
    use crate::codebase::ts_source::facts::{
        collect_file_facts_from_program, TsFactContext, TsFactPlan,
    };
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/module-captures/pure-import.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let (facts, ts) = crate::ast::with_program(&path, &source, |program, _| {
        (
            query_annotation::collect(
                program,
                &source,
                &EmbeddedSqlOptions::configured("@example/db", &[]),
            ),
            collect_file_facts_from_program(
                &path,
                TsFactPlan {
                    imports: true,
                    module_bindings: true,
                    ..Default::default()
                },
                &TsFactContext::default(),
                &source,
                program,
                None,
                None,
            ),
        )
    })
    .unwrap();
    let module = Arc::new(Module::new(Arc::new(ts)));
    let files = FxHashMap::from_iter([(
        path.clone(),
        File {
            facts: &facts,
            module: Arc::clone(&module),
            executors: Default::default(),
            initial_template: Default::default(),
        },
    )]);
    let resolutions = std::cell::Cell::new(0);
    for cached in [false, true] {
        if cached {
            assert!(module
                .target("./pure.mts", &path, &|_, _| {
                    resolutions.set(resolutions.get() + 1);
                    Some(path.with_file_name("pure.mts"))
                })
                .is_some());
        }
        let mut isolated =
            super::super::new::without_resolver::<fn(&str, &Path) -> Option<PathBuf>>(&files);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            isolated.name(&path, "statement", 16, true)
        }))
        .err()
        .expect("even a cached import lookup requires a resolver capability");
        assert_eq!(
            panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied()),
            Some("pure template cannot resolve imports")
        );
        assert_eq!(resolutions.get(), usize::from(cached));
        // Canonical inert construction itself never reads the import. Its
        // function body remains unevaluated, including after target caching.
        assert!(construct::<fn(&str, &Path) -> Option<PathBuf>>(&files, &path, None).is_some());
        assert_eq!(resolutions.get(), usize::from(cached));
    }
}
