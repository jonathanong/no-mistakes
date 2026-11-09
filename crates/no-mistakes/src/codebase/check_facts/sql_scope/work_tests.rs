macro_rules! reject_broad_control {
    () => {
        if $crate::codebase::check_facts::sql_scope::tests::BROAD.with(std::cell::Cell::get) {
            return false;
        }
    };
}

macro_rules! record_seed_check {
    () => {
        if let Some(observer) = $crate::diagnostics::current() {
            observer.increment("sql_scope.seed_checks", 1);
        }
    };
}

macro_rules! record_sql_delegation {
    () => {
        if let Some(observer) = $crate::diagnostics::current() {
            observer.increment("sql_scope.single_delegations", 1);
        }
    };
}
