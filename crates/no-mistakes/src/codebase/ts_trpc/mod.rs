pub mod calls {
    pub use no_mistakes_ts_analysis::ts_trpc::calls::*;

    #[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
    mod tests;
}
pub use calls::TrpcCallFact;
pub(crate) use calls::{finish_trpc_calls, procedure_path_from_call};
pub use no_mistakes_ts_analysis::ts_trpc::router;
pub use router::extract_trpc_router_from_program;
