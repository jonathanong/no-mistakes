fn is_elixir_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.elixir.apps)
        && (rel.ends_with("mix.exs") || (rel.ends_with(".ex") && !is_elixir_test(rel)))
        || is_named_manifest(root, &config.tests.elixir.apps, rel, "mix.exs")
}

fn is_elixir_test(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with("_test.exs")
        || (rel.contains("/test/") || rel.starts_with("test/"))
            && (name.ends_with(".ex") || name.ends_with(".exs"))
}
