use super::*;
use crate::codebase::{
    analysis_session::AnalysisSession,
    check_facts::{
        collect_check_facts,
        file::{collect_file_fact_variants_with_session, CheckFactVariant},
        CheckFactPlan,
    },
};

fn fixture() -> (PathBuf, CheckFactPlan) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/sql-helper-prerequisite")
        .canonicalize()
        .unwrap();
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("membership.yml"))).unwrap();
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &[],
        &crate::codebase::rules::source_store_for_files(&[]),
    )
    .unwrap();
    let mut plan = CheckFactPlan {
        imports: true,
        ..Default::default()
    };
    crate::codebase::postgres::configure_prepared_postgres_plan(&root, &config, &mut plan).unwrap();
    assert_eq!(
        planned_low_applications(&root, &root.join("helper.ts"), &plan).len(),
        1,
        "saved fixture must prepare one low application for helper.ts"
    );
    plan.query_annotation = true;
    plan.query_annotation_options = Some(vec![EmbeddedSqlOptions::configured("@selected/db", &[])]);
    (root, plan)
}

fn assert_membership(files: &FileIdMap<CheckFileFacts>, path: &Path) {
    assert!(source_failed(files, path));
    for (module, wanted) in [("@selected/db", true), ("@other/db", false)] {
        let profile = EmbeddedSqlOptions::configured(module, &[]);
        let index = usize::from(!wanted);
        let low = SqlSourceKey::application(
            SqlProjection::LowSyntax,
            &profile,
            "postgres-no-offset",
            index,
        );
        assert_eq!(
            source_demanded(files, path, &low),
            wanted,
            "path={}, key={low:?}, retained={:?}",
            path.display(),
            files.get(path).unwrap().sql_source_prerequisites
        );
        let max = SqlSourceKey::profile(SqlProjection::AnnotationMax, &profile);
        assert_eq!(
            source_demanded(files, path, &max),
            wanted,
            "path={}, key={max:?}, retained={:?}",
            path.display(),
            files.get(path).unwrap().sql_source_prerequisites
        );
    }
    assert_eq!(
        files
            .get(path)
            .unwrap()
            .sql_identifier_syntax
            .low_applications
            .len(),
        1
    );
}

#[test]
fn failed_parse_and_read_preserve_planned_profile_and_application_membership() {
    let (root, plan) = fixture();
    for name in ["helper.ts", "missing.ts"] {
        let path = root.join(name);
        let facts = collect_check_facts(&root, vec![path.clone()], plan.clone());
        let files = FileIdMap::from_entries([(
            path.clone(),
            CheckFileFacts {
                parse_error: facts.ts[&path].parse_error.clone(),
                sql_source_prerequisites: facts.ts[&path].sql_source_prerequisites.clone(),
                sql_identifier_syntax:
                    crate::codebase::postgres::sql_identifier_syntax::FileFacts {
                        low_applications: facts.ts[&path]
                            .sql_identifier_syntax
                            .low_applications
                            .clone(),
                        ..Default::default()
                    },
                ..Default::default()
            },
        )]);
        assert_membership(&files, &path);
        let session = AnalysisSession::disabled();
        let variant = CheckFactVariant {
            root: &root,
            plan: &plan,
            playwright: None,
        };
        let file = collect_file_fact_variants_with_session(&session, &path, &[variant])
            .pop()
            .unwrap()
            .unwrap();
        assert_membership(&FileIdMap::from_entries([(path.clone(), file)]), &path);
    }
}

#[test]
fn explicit_empty_demand_and_legacy_membership_have_distinct_meaning() {
    let (root, _) = fixture();
    let path = root.join("helper.ts");
    let profile = EmbeddedSqlOptions::configured("@selected/db", &[]);
    let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, &profile);
    let mut files = FileIdMap::from_entries([(path.clone(), CheckFileFacts::default())]);
    assert!(source_demanded(&files, &path, &key));
    files.get_mut(&path).unwrap().sql_source_prerequisites =
        SqlSourcePrerequisites::failed_source(&root, &path, &CheckFactPlan::default());
    assert!(!source_demanded(&files, &path, &key));
    assert!(source_demanded(
        &files,
        &path,
        &SqlSourceKey::profile(SqlProjection::Embedded, &profile)
    ));
}

#[test]
fn unsupported_missing_source_is_not_a_sql_catalog_prerequisite() {
    let (root, plan) = fixture();
    let path = root.join("missing.json");
    let session = AnalysisSession::disabled();
    let variant = CheckFactVariant {
        root: &root,
        plan: &plan,
        playwright: None,
    };
    let file = collect_file_fact_variants_with_session(&session, &path, &[variant])
        .pop()
        .unwrap()
        .unwrap();
    let files = FileIdMap::from_entries([(path.clone(), file)]);
    assert!(source_failed(&files, &path));
    for module in ["@selected/db", "@other/db"] {
        let profile = EmbeddedSqlOptions::configured(module, &[]);
        assert!(!source_demanded(
            &files,
            &path,
            &SqlSourceKey::profile(SqlProjection::AnnotationMax, &profile)
        ));
        assert!(!source_demanded(
            &files,
            &path,
            &SqlSourceKey::application(SqlProjection::LowSyntax, &profile, "postgres-no-offset", 0)
        ));
    }
}

#[test]
fn unsupported_saved_source_failure_keeps_an_explicit_empty_sql_catalog() {
    let (root, plan) = fixture();
    let path = root.join("membership.yml");
    let facts = collect_check_facts(&root, vec![path.clone()], plan.clone());
    assert!(facts.ts.get(&path).is_none());
    let session = AnalysisSession::disabled();
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let file = crate::codebase::check_facts::file::collect_file_facts_with_session_and_sources(
        &session, &root, &path, &plan, None, &sources, false,
    )
    .unwrap();
    assert!(file.parse_error.is_some());
    let declared = file
        .sql_source_prerequisites
        .declared
        .as_ref()
        .expect("unsupported source retains explicit planned eligibility");
    assert!(
        declared.is_empty(),
        "path={}, declared={declared:?}",
        path.display()
    );
}

#[test]
fn legacy_direct_plans_retain_default_and_explicit_embedded_profiles_on_failure() {
    let (root, _) = fixture();
    let path = root.join("helper.ts");
    let first = EmbeddedSqlOptions::configured("@selected/db", &[]);
    let second = EmbeddedSqlOptions::configured("@other/db", &[]);
    for (embedded, annotation, expected) in [
        (Vec::new(), None, vec![EmbeddedSqlOptions::default()]),
        (
            vec![first.clone(), second.clone()],
            None,
            vec![first.clone(), second.clone()],
        ),
        (
            vec![first.clone(), second.clone()],
            Some(vec![first.clone()]),
            vec![first.clone()],
        ),
    ] {
        let plan = CheckFactPlan {
            embedded_sql: true,
            embedded_sql_options: embedded,
            query_annotation: true,
            query_annotation_options: annotation,
            ..Default::default()
        };
        let facts = collect_check_facts(&root, vec![path.clone()], plan);
        assert!(facts.ts[&path].parse_error.is_some());
        let file = CheckFileFacts {
            parse_error: facts.ts[&path].parse_error.clone(),
            sql_source_prerequisites: facts.ts[&path].sql_source_prerequisites.clone(),
            ..Default::default()
        };
        let files = FileIdMap::from_entries([(path.clone(), file)]);
        for profile in [EmbeddedSqlOptions::default(), first.clone(), second.clone()] {
            let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, &profile);
            assert_eq!(
                source_demanded(&files, &path, &key),
                expected.contains(&profile),
                "path={}, key={key:?}, expected={expected:?}",
                path.display()
            );
        }
    }
}
