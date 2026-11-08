fn initial_sites(
    sites: &[ResolvedCallSite],
    admitted: impl Iterator<Item = usize>,
) -> (Vec<usize>, Vec<ResolvedCallSite>) {
    let admitted: Vec<_> = admitted.collect();
    let originals = admitted
        .iter()
        .map(|index| {
            #[cfg(all(test, feature = "test-instrumentation"))]
            initial_sites_tests::record_clone();
            sites[*index].clone()
        })
        .collect();
    (admitted, originals)
}
