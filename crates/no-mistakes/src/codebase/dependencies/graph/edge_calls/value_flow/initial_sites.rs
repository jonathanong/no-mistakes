#[cfg(test)]
thread_local! {
    static INITIAL_SITE_CLONES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn initial_sites(
    sites: &[ResolvedCallSite],
    admitted: impl Iterator<Item = usize>,
) -> (Vec<usize>, Vec<ResolvedCallSite>) {
    let admitted: Vec<_> = admitted.collect();
    let originals = admitted
        .iter()
        .map(|index| {
            #[cfg(test)]
            INITIAL_SITE_CLONES.set(INITIAL_SITE_CLONES.get() + 1);
            sites[*index].clone()
        })
        .collect();
    (admitted, originals)
}
