pub(crate) fn collect(
    root: &Path,
    path: &Path,
    program: &Program<'_>,
    source: &str,
    imports: &[ImportedBinding],
    config: &RouteCoverageSource,
    resolution: &PlaywrightModuleResolution,
) -> Vec<RouteOccurrence> {
    let receivers = Receivers::collect(program);
    let mut visitor = Collector {
        root, path, source, imports, config, resolution, receivers: &receivers,
        test: None, describes: Vec::new(), occurrences: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.occurrences.sort();
    visitor.occurrences.dedup();
    visitor.occurrences
}

struct Collector<'a, 'p> {
    root: &'p Path,
    path: &'p Path,
    source: &'a str,
    imports: &'p [ImportedBinding],
    config: &'p RouteCoverageSource,
    resolution: &'p PlaywrightModuleResolution,
    receivers: &'p Receivers,
    test: Option<String>,
    describes: Vec<String>,
    occurrences: Vec<RouteOccurrence>,
}
