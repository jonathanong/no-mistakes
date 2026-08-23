fn is_dart_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.dart.packages)
        && (rel.ends_with("pubspec.yaml") || (rel.ends_with(".dart") && !is_dart_test(rel)))
        || is_named_manifest(root, &config.tests.dart.packages, rel, "pubspec.yaml")
}

fn is_dart_test(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with("_test.dart")
        || (rel.contains("/test/") || rel.starts_with("test/")) && name.ends_with(".dart")
}
