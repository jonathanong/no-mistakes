#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod child {
    #[test]
    fn child_test() {}
}
