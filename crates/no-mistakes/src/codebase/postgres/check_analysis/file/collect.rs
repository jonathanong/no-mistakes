use super::{annotation, embedded, heads, sql_identifier, PostgresFileFacts};
use crate::codebase::check_facts::{
    collect_with_postgres, CheckFactPlan, CheckFactVariant, CheckFileFacts,
};
use crate::codebase::postgres_analysis::{FileInput, FilePayload, RecoveryPath};
use std::{path::Path, sync::Arc};

pub(crate) fn collect_program(
    variant: &CheckFactVariant<'_>,
    input: FileInput<'_, '_>,
    owned_source: Option<Arc<str>>,
) -> CheckFileFacts {
    let FileInput {
        root,
        path,
        source,
        program,
        plan,
    } = input;
    let mut scoped_executors = Default::default();
    let embedded_sql =
        embedded::prepared_with_scopes(root, path, source, program, plan, &mut scoped_executors);
    let mut head_candidates = heads::candidates(plan, &embedded_sql);
    let (helpers, helper_candidates) = heads::helper_candidates(plan, &embedded_sql, program);
    head_candidates.extend(helper_candidates);
    let loop_heads = (plan.query_annotation
        && !plan.query_annotation_heads
        && heads::has_annotation_executors(plan, &embedded_sql))
    .then(|| {
        crate::codebase::postgres::query_annotation::heads::prepare_loop_heads(
            program,
            source,
            !head_candidates.is_empty(),
            helpers,
        )
    })
    .flatten();
    let module_bindings = !head_candidates.is_empty() || loop_heads.is_some();
    collect_with_postgres(variant, input, owned_source, module_bindings, |ts| {
        let query_annotation =
            annotation::collect(root, path, source, program, plan, &embedded_sql);
        let query_annotation_heads =
            (plan.query_annotation_heads || !head_candidates.is_empty() || loop_heads.is_some())
                .then(|| {
                    Arc::new(
                        crate::codebase::postgres::query_annotation::heads::collect_prepared(
                            program,
                            ts,
                            heads::call_spans(plan, &embedded_sql),
                            &head_candidates,
                            plan.query_annotation_heads,
                            helpers,
                            loop_heads,
                        ),
                    )
                });
        let sql_identifier_syntax = sql_identifier::collect_with_scopes(
            root,
            path,
            source,
            program,
            ts,
            plan,
            &mut scoped_executors,
        );
        if embedded_sql.is_empty()
            && query_annotation.is_empty()
            && query_annotation_heads.is_none()
            && sql_identifier_syntax.is_none()
        {
            return FilePayload::new();
        }
        PostgresFileFacts {
            embedded_sql,
            query_annotation,
            query_annotation_heads,
            sql_identifier_syntax: sql_identifier_syntax.unwrap_or_default(),
        }
        .into_payload()
    })
}

pub(crate) fn recover_program(input: FileInput<'_, '_>, recovery: RecoveryPath) -> FilePayload {
    let FileInput {
        root,
        path,
        source,
        program,
        plan,
    } = input;
    let embedded = || embedded::prepared_embedded_sql(root, path, source, program, plan);
    let invalid = || sql_identifier::invalid(root, path, source, program, plan);
    let (embedded_sql, sql_identifier_syntax) = match recovery {
        RecoveryPath::Single => (embedded(), invalid()),
        RecoveryPath::Variants => {
            let syntax = invalid();
            (embedded(), syntax)
        }
    };
    PostgresFileFacts {
        embedded_sql,
        sql_identifier_syntax,
        ..Default::default()
    }
    .into_payload()
}

pub(crate) fn failed_source(root: &Path, path: &Path, plan: &CheckFactPlan) -> FilePayload {
    let low_applications =
        crate::codebase::check_facts::sql_source_prerequisites::planned_low_applications(
            root, path, plan,
        );
    if low_applications.is_empty() {
        return FilePayload::new();
    }
    PostgresFileFacts {
        sql_identifier_syntax: crate::codebase::postgres::sql_identifier_syntax::FileFacts {
            low_applications,
            ..Default::default()
        },
        ..Default::default()
    }
    .into_payload()
}
