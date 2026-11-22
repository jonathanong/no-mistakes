use super::{CheckFactPlan, FamilyPathScope};
use crate::codebase::analysis_session::AnalysisSession;
use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) struct FactRequest<'a> {
    pub(crate) session: &'a AnalysisSession,
    pub(crate) discovery: &'a FamilyPathScope,
    pub(super) projected_executor_scopes: Option<u32>,
    pub(super) batch_collected: bool,
}

impl<'a> FactRequest<'a> {
    pub(crate) fn prepared(session: &'a AnalysisSession, plan: &CheckFactPlan) -> Self {
        Self {
            session,
            projected_executor_scopes: None,
            batch_collected: false,
            discovery: if plan.integration {
                session.integration_fact_discovery()
            } else {
                &FamilyPathScope::All
            },
        }
    }

    pub(crate) fn collect_tests(self, plan: &CheckFactPlan, path: &Path) -> bool {
        !plan.integration || super::family_scope::applies(self.discovery, path)
    }
}

impl<'a> From<&'a AnalysisSession> for FactRequest<'a> {
    fn from(session: &'a AnalysisSession) -> Self {
        Self {
            session,
            projected_executor_scopes: None,
            batch_collected: false,
            discovery: &FamilyPathScope::All,
        }
    }
}

impl std::ops::Deref for FactRequest<'_> {
    type Target = AnalysisSession;
    fn deref(&self) -> &Self::Target {
        self.session
    }
}
