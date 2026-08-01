use super::*;

fn manifest_with(entries: &[(&str, &str)]) -> PackageManifest {
    let mut fields_by_name: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, field) in entries {
        fields_by_name
            .entry(name.to_string())
            .or_default()
            .insert(field.to_string());
    }
    PackageManifest { fields_by_name }
}

fn allowed(fields: &[&str]) -> BTreeSet<String> {
    fields.iter().map(|field| field.to_string()).collect()
}

#[test]
fn classify_reports_allowed_when_declared_under_an_allowed_field() {
    let manifest = manifest_with(&[("@acme/tool", "dependencies")]);
    assert_eq!(
        manifest.classify("@acme/tool", &allowed(&["dependencies"])),
        Classification::Allowed
    );
}

#[test]
fn classify_reports_dev_only_when_declared_only_outside_allowed_fields() {
    let manifest = manifest_with(&[("@acme/tool", "devDependencies")]);
    assert_eq!(
        manifest.classify("@acme/tool", &allowed(&["dependencies"])),
        Classification::DevOnly
    );
}

#[test]
fn classify_reports_allowed_when_declared_under_any_of_multiple_allowed_fields() {
    let manifest = manifest_with(&[("@acme/tool", "peerDependencies")]);
    assert_eq!(
        manifest.classify(
            "@acme/tool",
            &allowed(&["dependencies", "optionalDependencies", "peerDependencies"])
        ),
        Classification::Allowed
    );
}

#[test]
fn classify_reports_undeclared_for_absent_package() {
    let manifest = manifest_with(&[]);
    assert_eq!(
        manifest.classify("left-pad", &allowed(&["dependencies"])),
        Classification::Undeclared
    );
}

#[test]
fn load_reads_dependency_fields_from_a_real_manifest() {
    let root =
        crate::codebase::ts_resolver::normalize_path(&Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/production-dependency-declarations/dev-only-production-import",
        ));
    let manifest_path = root.join("packages/lib/package.json");
    let sources =
        crate::codebase::rules::source_store_for_files(std::slice::from_ref(&manifest_path));

    let manifest = PackageManifest::load(&manifest_path, &sources);

    assert_eq!(
        manifest.classify("@acme/tool", &allowed(&["dependencies"])),
        Classification::DevOnly
    );
}
