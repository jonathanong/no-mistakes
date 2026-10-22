macro_rules! record_call_site_collection {
    () => {
        if let Some(observer) = crate::diagnostics::current() {
            observer.increment("call_sites.collect_files", 1);
        }
    };
}
