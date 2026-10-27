macro_rules! probe {
    () => {
        tests::probe()
    };
}
macro_rules! extend_origins {
    ($found:expr, $origins:expr) => {
        $found.pending_env.extend($origins.values().map(|origin| {
            tests::origin_visit();
            *origin
        }));
    };
}
macro_rules! legacy_dispatch {
    ($found:expr, $base:expr, $mapped:expr, $captured:expr) => {
        if tests::legacy_enabled() {
            let old = tests::legacy_prefix($base, $mapped, $captured);
            $found.pending_env.extend(old.pending_env);
            $found.pending_args.extend(old.pending_args);
            return;
        }
    };
}
