use super::{CheckFactPlan, FamilyPathScope};
use crate::codebase::analysis_session::AnalysisSession;
use std::path::Path;

#[derive(Clone, Copy)]
pub(crate) struct FactRequest<'a> {
    pub(crate) session: &'a AnalysisSession,
    pub(crate) discovery: &'a FamilyPathScope,
}

impl<'a> FactRequest<'a> {
    pub(crate) fn prepared(session: &'a AnalysisSession, plan: &CheckFactPlan) -> Self {
        Self {
            session,
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
