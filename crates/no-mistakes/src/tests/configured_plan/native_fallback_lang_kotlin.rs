fn is_kotlin_native(rel: &str, root: &Path, config: &NoMistakesConfig) -> bool {
    under_roots(rel, &config.tests.kotlin.packages)
        && (rel.ends_with("build.gradle")
            || rel.ends_with("build.gradle.kts")
            || (rel.ends_with(".kt") && !is_kotlin_test(rel)))
        || is_named_manifest(
            root,
            &config.tests.kotlin.packages,
            rel,
            "build.gradle.kts",
        )
}

fn is_kotlin_test(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with("Test.kt")
        || name.ends_with("Tests.kt")
        || name.ends_with("IT.kt")
        || (rel.contains("/src/test/") || rel.starts_with("src/test/")) && name.ends_with(".kt")
}
