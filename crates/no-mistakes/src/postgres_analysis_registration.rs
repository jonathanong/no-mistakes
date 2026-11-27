use crate::codebase::postgres::{
    self,
    check_analysis::{self, file, relative_scope},
};
use crate::codebase::postgres_analysis::{self, Hooks};

ctor::declarative::ctor! {
    // SAFETY: Registration stores a reference to an immutable static table.
    #[ctor(unsafe)]
    fn register_postgres_analysis() {
        postgres_analysis::register(&HOOKS);
    }
}

static HOOKS: Hooks = Hooks {
    collect_program: file::collect_program,
    retain_source: file::policy_selected,
    recover_program: file::recover_program,
    failed_source: file::failed_source,
    project: relative_scope::project_relative_executor_scopes,
    prepare: check_analysis::prepare,
    count_unproven: relative_scope::count_unproven_in_view,
    needs_import_support: file::needs_import_support,
    configure_profiles: check_analysis::configure_profiles,
    configure_plan: postgres::configure_prepared_postgres_plan,
    configure_helper_scope: postgres::configure_lock_inventory_helper_call_sites,
    has_sql_advice: postgres::has_sql_advice,
};
