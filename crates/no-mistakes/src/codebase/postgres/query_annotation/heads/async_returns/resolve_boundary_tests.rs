//! Prepared partial maps preserve advisories instead of inventing async origins.
use super::{resolve, Pending};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::query_annotation::heads::{AnnotationHeads, HeadProof};
use crate::codebase::ts_source::FileIdMap;
use std::{path::PathBuf, sync::Arc};
#[test]
fn async_projection_skips_headless_files_and_unresolved_factory_targets() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("async-return-partial/src/query.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let start = source.find("read(query)").unwrap() as u32;
    let span = (start, start + "read(query)".len() as u32);
    let mut heads = AnnotationHeads::default();
    heads.calls.insert(span, HeadProof::Unproven);
    heads.pending_async_returns.push(Pending {
        span,
        specifier: "./missing-factory.mjs".into(),
        imported: "makeQuery".into(),
    });
    let mut files = FileIdMap::from_entries([
        (
            path.clone(),
            CheckFileFacts {
                query_annotation_heads: Some(Arc::new(heads)),
                ..Default::default()
            },
        ),
        (PathBuf::from("headless.mts"), CheckFileFacts::default()),
    ]);
    let resolutions = std::cell::Cell::new(0);
    resolve::project(&mut files, &|specifier, from| {
        assert_eq!(specifier, "./missing-factory.mjs");
        assert_eq!(from, path.as_path());
        resolutions.set(resolutions.get() + 1);
        None
    });
    assert_eq!(resolutions.get(), 1);
    assert_eq!(
        files
            .get(&path)
            .unwrap()
            .query_annotation_heads
            .as_ref()
            .unwrap()
            .calls[&span],
        HeadProof::Unproven
    );
    assert!(files
        .get(std::path::Path::new("headless.mts"))
        .unwrap()
        .query_annotation_heads
        .is_none());
}
