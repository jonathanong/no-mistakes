fn source_file_matches_filter(root: &Path, path: &Path, filter: Option<&GlobSet>) -> bool {
    filter
        .map(|filter| filter.is_match(path.strip_prefix(root).unwrap_or(path)))
        .unwrap_or(true)
}

fn build_filter(filters: &[String]) -> anyhow::Result<Option<GlobSet>> {
    if filters.is_empty() {
        return Ok(None);
    }
    let mut builder = GlobSetBuilder::new();
    for filter in filters {
        builder.add(GlobBuilder::new(filter).literal_separator(false).build()?);
    }
    Ok(Some(builder.build()?))
}
