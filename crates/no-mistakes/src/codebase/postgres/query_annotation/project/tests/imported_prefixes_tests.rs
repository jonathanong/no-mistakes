use super::*;
use crate::codebase::postgres::TrustedSqlTag;

#[test]
fn imported_prefixes_reuse_resolved_tags_and_require_matching_origins() {
    let mut files = prepared_fixture_with_profiles(
        "imported-callback-prefix",
        &[
            "callers",
            "db",
            "guards",
            "other/source",
            "other/sql",
            "other-sql",
            "pager",
            "sql",
        ],
        &[
            EmbeddedSqlOptions::configured("./db.mjs", &[]).with_trusted_sql_tags(
                &["./sql.mjs", "././sql.mjs", "./other-sql.mjs"].map(|module| TrustedSqlTag {
                    module: module.into(),
                    name: "sql".into(),
                }),
            ),
        ],
    );
    let paths = files.keys().cloned().collect::<BTreeSet<_>>();
    let root = paths
        .iter()
        .find(|path| path.ends_with("callers.mts"))
        .unwrap()
        .parent()
        .unwrap();
    let resolutions = Mutex::new(BTreeMap::new());
    project(&mut files, |specifier, from| {
        *resolutions
            .lock()
            .unwrap()
            .entry((from.to_path_buf(), specifier.to_string()))
            .or_insert(0) += 1;
        let relative = specifier.strip_prefix("./")?;
        let target = from.parent()?.join(relative).with_extension("mts");
        paths.get(&target).cloned()
    });
    let resolutions = resolutions.into_inner().unwrap();
    for (owner, specifier) in [
        ("callers.mts", "./sql.mjs"),
        ("callers.mts", "././sql.mjs"),
        ("pager.mts", "./sql.mjs"),
        ("other/source.mts", "./sql.mjs"),
    ] {
        assert_eq!(resolutions[&(root.join(owner), specifier.into())], 1);
    }
    assert!(resolutions.values().all(|count| *count == 1));

    for (file, declaration, expected) in [
        (
            "callers.mts",
            "first",
            Some("/* first */ SELECT id FROM widgets WHERE true"),
        ),
        (
            "callers.mts",
            "alias",
            Some("/* alias */ SELECT id FROM widgets"),
        ),
        ("guards.mts", "relativeForeign", None),
    ] {
        let file = &files[&root.join(file)];
        let source = file.source.as_deref().unwrap();
        let declaration = source
            .find(&format!("export const {declaration} "))
            .unwrap();
        let call = "read(statement)";
        let start = (declaration + source[declaration..].find(call).unwrap()) as u32;
        let span = (start, start + call.len() as u32);
        let facts = &file.postgres().query_annotation[0].1;
        assert_eq!(facts.calls[&span].as_deref(), expected);
        assert_eq!(
            facts.proven_call_prefixes.contains(&span),
            expected.is_some()
        );
    }
}
