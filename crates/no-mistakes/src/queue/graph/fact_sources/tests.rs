use super::*;
use crate::codebase::ts_source::facts::TsFileFacts;
use globset::{Glob, GlobSetBuilder};

#[test]
fn ts_fact_conversion_applies_filter_before_taking_queue_facts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let included = root.join("src/included.ts");
    let excluded = root.join("src/excluded.ts");
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("src/included.ts").unwrap());
    let filter = builder.build().unwrap();
    let ts_facts = crate::codebase::ts_source::facts::TsFactMap::from([
        (
            included.clone(),
            TsFileFacts {
                queue_project: Some(FileFacts::default()),
                ..Default::default()
            },
        ),
        (
            excluded,
            TsFileFacts {
                queue_project: Some(FileFacts::default()),
                ..Default::default()
            },
        ),
    ]);

    let facts = queue_project_facts_from_ts(ts_facts, Some(&filter), &root);

    assert_eq!(facts.len(), 1);
    assert!(facts.contains_key(&included));
}
