use super::callbacks::CallbackWalk;
use super::receivers::FileEnv;
use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::ResolvedCallSite;

impl CallbackWalk<'_> {
    pub(super) fn site_is_deferred(
        &self,
        env: &FileEnv<'_>,
        site: &ResolvedCallSite,
        tx_owner: CallableId,
    ) -> bool {
        self.defer_async
            && site.caller_id.is_some_and(|caller| {
            caller != tx_owner
                && (self.deferred.contains(&caller)
                    || env
                        .file_facts
                        .value_flow
                        .scoped_handles
                        .first_await_ends
                        .get(&caller)
                        .is_some_and(|end| *end <= site.offset))
        })
    }

    pub(super) fn report_deferred(&mut self, env: &FileEnv<'_>, site: &ResolvedCallSite) {
        self.report_unanalyzable(env, site,
            "publish inside an async built-in callback may run after the transaction closes; move the publish after commit");
    }
}
