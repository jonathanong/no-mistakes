use super::super::{
    query_annotation, sql_identifier_syntax, EmbeddedSqlFileFacts, EmbeddedSqlOptions,
};
use crate::codebase::check_facts::{CheckFactPlan, CheckFileFacts, SqlSourcePrerequisites};
use crate::codebase::postgres_analysis::FilePayload;
use std::{
    path::Path,
    sync::{Arc, LazyLock},
};

mod collect;
pub(crate) use collect::{collect_program, failed_source, recover_program};
#[path = "file/program_annotation.rs"]
mod annotation;
#[path = "file/program_embedded.rs"]
pub(crate) mod embedded;
#[path = "file/program_heads.rs"]
mod heads;
mod sql_identifier;
pub(crate) use sql_identifier::policy_selected;

#[derive(Default)]
pub(crate) struct PostgresFileFacts {
    pub(crate) embedded_sql: Vec<(EmbeddedSqlOptions, EmbeddedSqlFileFacts)>,
    pub(crate) query_annotation: Vec<(
        EmbeddedSqlOptions,
        query_annotation::QueryAnnotationFileFacts,
    )>,
    pub(crate) query_annotation_heads: Option<Arc<query_annotation::heads::AnnotationHeads>>,
    pub(crate) sql_identifier_syntax: sql_identifier_syntax::FileFacts,
}

impl PostgresFileFacts {
    pub(crate) fn into_payload(self) -> FilePayload {
        std::sync::OnceLock::from(Box::new(self) as Box<dyn std::any::Any + Send + Sync>)
    }
}

static EMPTY: LazyLock<PostgresFileFacts> = LazyLock::new(PostgresFileFacts::default);

impl CheckFileFacts {
    pub(crate) fn postgres_if_present(&self) -> Option<&PostgresFileFacts> {
        self.postgres_file
            .get()
            .map(|facts| facts.downcast_ref().expect("Postgres file fact payload"))
    }

    pub(crate) fn postgres(&self) -> &PostgresFileFacts {
        self.postgres_if_present().unwrap_or(&EMPTY)
    }

    // A policy may populate lazy caches, so it needs the file-owned payload.
    pub(crate) fn postgres_for_policy(&self) -> &PostgresFileFacts {
        self.postgres_file
            .get_or_init(|| Box::<PostgresFileFacts>::default())
            .downcast_ref()
            .expect("Postgres file fact payload")
    }

    pub(crate) fn postgres_and_source_prerequisites(
        &mut self,
    ) -> (&PostgresFileFacts, &mut SqlSourcePrerequisites) {
        let postgres = self.postgres_file.get().map_or(&*EMPTY, |facts| {
            facts.downcast_ref().expect("Postgres file fact payload")
        });
        (postgres, &mut self.sql_source_prerequisites)
    }

    pub(crate) fn postgres_if_present_mut(&mut self) -> Option<&mut PostgresFileFacts> {
        self.postgres_file
            .get_mut()
            .map(|facts| facts.downcast_mut().expect("Postgres file fact payload"))
    }

    pub(crate) fn postgres_mut(&mut self) -> &mut PostgresFileFacts {
        self.postgres_file
            .get_or_init(|| Box::<PostgresFileFacts>::default());
        self.postgres_file
            .get_mut()
            .unwrap()
            .downcast_mut()
            .expect("Postgres file fact payload")
    }
}

pub(crate) fn needs_import_support(file: &CheckFileFacts) -> bool {
    file.postgres_if_present().is_some_and(|file| {
        file.embedded_sql.iter().any(|(_, facts)| {
            !facts.pending_imported.is_empty()
                || !facts.imported_lock_helpers.is_empty()
                || !facts.pending_relative.candidates.is_empty()
        })
    })
}
