use super::super::{
    resolve_pnpm_include, resolve_pnpm_include_at_root, PackageGlobs, PnpmInclude,
    ReleaseAgeExclude,
};
use std::path::Path;
use std::sync::Arc;

fn resolve(yaml: &str, files: &[(&str, &str)]) -> PnpmInclude {
    resolve_pnpm_include(Path::new("/repo"), yaml, |path| {
        let text = path.to_string_lossy();
        files
            .iter()
            .find(|(name, _)| text.ends_with(name))
            .map(|(_, body)| Arc::<str>::from(*body))
    })
    .unwrap()
}

fn list(include: &PnpmInclude) -> &[String] {
    match &include.packages {
        PackageGlobs::List(list) => list,
        PackageGlobs::DefaultStar => panic!("expected a package list, got the omitted default"),
        PackageGlobs::Conflict => panic!("expected a package list, got a conflict"),
    }
}

#[test]
fn omitted_packages_without_include_stay_the_default_star() {
    let include = resolve("catalog: {}\n", &[]);
    assert!(matches!(include.packages, PackageGlobs::DefaultStar));
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::FromRoot
    ));
    let include = resolve("packages: null\n", &[]);
    assert!(matches!(include.packages, PackageGlobs::DefaultStar));
}

#[test]
fn root_package_list_is_kept_verbatim() {
    let include = resolve("packages: [packages/*, '!packages/hidden']\n", &[]);
    assert_eq!(
        list(&include),
        ["packages/*".to_string(), "!packages/hidden".to_string()]
    );
    assert!(!include.packages_invalid);
    let include = resolve("minimumReleaseAgeExclude: [acme-lib]\n", &[]);
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::FromRoot
    ));
}

#[test]
fn included_package_globs_are_rebased_onto_the_file() {
    let include = resolve(
        "include:\n  ./pnpm/packages.yaml: [packages]\n",
        &[(
            "/pnpm/packages.yaml",
            "packages: ['../packages/*', './nested/*', '!../packages/hidden', '/abs/*', '~/x', '$y', '../../nope/*']\n",
        )],
    );
    assert_eq!(
        list(&include),
        [
            "packages/*".to_string(),
            "pnpm/nested/*".to_string(),
            "!packages/hidden".to_string(),
        ]
    );
}

#[test]
fn config_dependency_sources_are_ignored() {
    let include = resolve(
        "include:\n  '@acme/pnpm-policy': [packages]\n",
        &[(
            "/node_modules/@acme/pnpm-policy/pnpm-workspace.yaml",
            "packages: [services/*]\n",
        )],
    );
    assert!(matches!(include.packages, PackageGlobs::DefaultStar));
    assert!(include.load_errors.is_empty());
    assert!(include.duplicates.is_empty());
}

#[test]
fn delegated_packages_that_are_omitted_do_not_become_star() {
    let include = resolve(
        "include:\n  ./pnpm/packages.yaml: [packages]\n",
        &[("/pnpm/packages.yaml", "catalog: {}\n")],
    );
    assert_eq!(list(&include), &[] as &[String]);
    let include = resolve("include:\n  ./pnpm/missing.yaml: [packages]\n", &[]);
    assert_eq!(list(&include), &[] as &[String]);
    assert!(include.load_errors[0].affects("packages"));
    assert!(!include.load_errors[0].affects("overrides"));
}

#[test]
fn duplicate_settings_do_not_pick_a_winner() {
    let include = resolve(
        "packages: [packages/*]\ninclude:\n  ./pnpm/packages.yaml: [packages]\n  ./pnpm/security.yaml: [overrides]\noverrides:\n  lodash: 1.0.0\n",
        &[
            ("/pnpm/packages.yaml", "packages: ['../services/*']\n"),
            ("/pnpm/security.yaml", "overrides:\n  react: 18.0.0\n"),
        ],
    );
    assert!(matches!(include.packages, PackageGlobs::Conflict));
    assert_eq!(
        include.duplicates,
        vec!["overrides".to_string(), "packages".to_string()]
    );
    assert_eq!(include.included_override_paths.len(), 1);
    assert!(include.included_override_paths[0].ends_with("pnpm/security.yaml"));
}

#[test]
fn star_grant_owns_only_settings_the_file_defines() {
    let include = resolve(
        "include:\n  ./pnpm/all.yaml: '*'\n",
        &[("/pnpm/all.yaml", "overrides:\n  lodash: 1.0.0\n")],
    );
    assert!(matches!(include.packages, PackageGlobs::DefaultStar));
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::FromRoot
    ));
    assert_eq!(include.included_override_paths.len(), 1);
    let include = resolve(
        "overrides:\n  lodash: 1.0.0\ninclude:\n  ./pnpm/all.yaml: '*'\n",
        &[("/pnpm/all.yaml", "overrides:\n  react: 18.0.0\n")],
    );
    assert!(include
        .duplicates
        .iter()
        .any(|setting| setting == "overrides"));
    assert_eq!(include.included_override_paths.len(), 1);
}

#[test]
fn unreadable_star_reserves_every_setting() {
    let include = resolve("include:\n  ./pnpm/all.yaml: '*'\n", &[]);
    assert_eq!(list(&include), &[] as &[String]);
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::Included(values) if values.is_empty()
    ));
    assert!(include.load_errors[0].affects("minimumReleaseAgeExclude"));
}

#[test]
fn invalid_local_sources_reserve_their_grant() {
    let mut seen = Vec::new();
    let include = resolve_pnpm_include(
        Path::new("/repo"),
        "include:\n  ./pnpm/packages.txt: [packages]\n  ../outside.yaml: [minimumReleaseAgeExclude]\n  ./has*.yaml: [overrides]\n",
        |path| {
            seen.push(path.display().to_string());
            None
        },
    )
    .unwrap();
    assert!(seen.is_empty(), "{seen:?}");
    assert_eq!(list(&include), &[] as &[String]);
    assert!(include
        .load_errors
        .iter()
        .any(|issue| issue.message.contains("not a local YAML file")));
    assert!(include
        .load_errors
        .iter()
        .any(|issue| issue.message.contains("escapes the workspace root")));
}

#[test]
fn include_path_outside_the_workspace_is_not_read() {
    let include = resolve_pnpm_include(
        Path::new("/repo"),
        "include:\n  ../outside.yaml: [packages]\n",
        |_| panic!("escaped include must not be read"),
    )
    .unwrap();
    assert_eq!(list(&include), &[] as &[String]);
    assert!(include.load_errors[0].message.contains("escapes"));
}

#[test]
fn nested_include_is_not_followed() {
    let mut seen = Vec::new();
    let include = resolve_pnpm_include(
        Path::new("/repo"),
        "include:\n  ./pnpm/a.yaml: [packages]\n",
        |path| {
            seen.push(path.display().to_string());
            Some(Arc::from(
                "include:\n  ./b.yaml: [packages]\npackages: ['../packages/*']\n",
            ))
        },
    )
    .unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(list(&include), ["packages/*".to_string()]);
}

#[test]
fn yml_extension_and_bad_included_documents_are_reserved() {
    let include = resolve(
        "include:\n  ./pnpm/packages.yml: [packages]\n",
        &[("/pnpm/packages.yml", "packages: ['../packages/app']\n")],
    );
    assert_eq!(list(&include), ["packages/app".to_string()]);
    let include = resolve(
        "include:\n  ./pnpm/bad.yaml: [packages]\n  ./pnpm/list.yaml: [minimumReleaseAgeExclude]\n  ./pnpm/text.yaml: [overrides]\n",
        &[
            ("/pnpm/bad.yaml", ":\n"),
            ("/pnpm/list.yaml", "- nope\n"),
            ("/pnpm/text.yaml", "overrides: nope\n"),
        ],
    );
    assert_eq!(list(&include), &[] as &[String]);
    assert!(include
        .load_errors
        .iter()
        .any(|issue| issue.message.contains("not valid YAML")));
    assert!(include
        .load_errors
        .iter()
        .any(|issue| issue.message.contains("must be a mapping")));
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::Included(values) if values.is_empty()
    ));
}

#[test]
fn invalid_grants_and_non_mapping_include_fail_closed() {
    let include = resolve(
        "include:\n  ./pnpm/packages.yaml: [packages, 1]\n",
        &[("/pnpm/packages.yaml", "packages: ['../packages/*']\n")],
    );
    assert_eq!(list(&include), &[] as &[String]);
    assert!(include.load_errors[0].affects("overrides"));
    let include = resolve("include:\n  - ./pnpm/packages.yaml\n", &[]);
    assert_eq!(list(&include), &[] as &[String]);
    assert!(include.load_errors[0].message.contains("must be a mapping"));
    let include = resolve("include:\n  ? [1]\n  : [packages]\n", &[]);
    assert!(matches!(include.packages, PackageGlobs::DefaultStar));
    assert!(resolve_pnpm_include(Path::new("/repo"), "- packages\n", |_| None).is_err());
    assert!(resolve_pnpm_include(Path::new("/repo"), ":\n", |_| None).is_err());
}

#[test]
fn root_packages_of_the_wrong_type_are_invalid() {
    let include = resolve("packages: nope\n", &[]);
    assert!(include.packages_invalid);
    assert_eq!(list(&include), &[] as &[String]);
    let include = resolve(
        "include:\n  ./pnpm/packages.yaml: [packages]\n",
        &[("/pnpm/packages.yaml", "packages: nope\n")],
    );
    assert!(!include.packages_invalid);
    assert_eq!(list(&include), &[] as &[String]);
}

#[test]
fn release_age_comes_from_the_owning_file() {
    let include = resolve(
        "include:\n  ./pnpm/age.yaml: [minimumReleaseAgeExclude]\n",
        &[(
            "/pnpm/age.yaml",
            "minimumReleaseAgeExclude: [acme-lib, 1]\n",
        )],
    );
    let ReleaseAgeExclude::Included(values) = &include.release_age_exclude else {
        panic!("expected included entries");
    };
    assert_eq!(values[0].as_str(), Some("acme-lib"));
    assert!(values[1].as_str().is_none());
    let include = resolve(
        "minimumReleaseAgeExclude: [acme-lib]\ninclude:\n  ./pnpm/age.yaml: [minimumReleaseAgeExclude]\n",
        &[("/pnpm/age.yaml", "minimumReleaseAgeExclude: [other]\n")],
    );
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::Conflict
    ));
    let include = resolve(
        "include:\n  ./pnpm/age.yaml: [minimumReleaseAgeExclude]\n",
        &[("/pnpm/age.yaml", "minimumReleaseAgeExclude: nope\n")],
    );
    assert!(matches!(
        include.release_age_exclude,
        ReleaseAgeExclude::Included(values) if values.is_empty()
    ));
}

#[test]
fn catalogs_merge_from_the_root_and_from_local_includes() {
    let include = resolve(
        "catalog:\n  '@acme/domain': workspace:*\ncatalogs:\n  default:\n    '@acme/domain': workspace:^\n    '@acme/other': workspace:~\n  legacy:\n    '@acme/legacy': link:./legacy\n",
        &[],
    );
    let domain = include.catalogs.get("", "@acme/domain").unwrap();
    assert_eq!(domain.specifier, "workspace:*");
    assert_eq!(
        include
            .catalogs
            .get("default", "@acme/other")
            .unwrap()
            .specifier,
        "workspace:~"
    );
    let legacy = include.catalogs.get("legacy", "@acme/legacy").unwrap();
    assert_eq!(legacy.specifier, "link:./legacy");
    assert_eq!(legacy.base_dir, Path::new("/repo"));
    let include = resolve(
        "include:\n  ./pnpm/catalog.yaml: [catalog]\n",
        &[(
            "/pnpm/catalog.yaml",
            "catalog:\n  '@acme/domain': workspace:../packages/domain\n",
        )],
    );
    let hit = include.catalogs.get("", "@acme/domain").unwrap();
    assert_eq!(hit.specifier, "workspace:../packages/domain");
    assert_eq!(hit.base_dir, Path::new("/repo/pnpm"));
}

#[test]
fn catalog_slots_claimed_twice_are_not_walked() {
    let include = resolve(
        "catalog:\n  '@acme/domain': workspace:*\ninclude:\n  ./pnpm/catalog.yaml: [catalog, catalogs.legacy]\ncatalogs:\n  legacy:\n    '@acme/legacy': workspace:*\n",
        &[(
            "/pnpm/catalog.yaml",
            "catalog:\n  '@acme/domain': workspace:^\ncatalogs:\n  legacy:\n    '@acme/legacy': workspace:~\n",
        )],
    );
    assert!(include.catalogs.get("", "@acme/domain").is_none());
    assert!(include.catalogs.get("legacy", "@acme/legacy").is_none());
    assert!(include
        .duplicates
        .iter()
        .any(|setting| setting == "catalog"));
    assert!(include
        .duplicates
        .iter()
        .any(|setting| setting == "catalogs.legacy"));
    let include = resolve(
        "catalogs:\n  legacy:\n    '@acme/legacy': workspace:*\ninclude:\n  ./pnpm/catalog.yaml: [catalogs]\n",
        &[("/pnpm/catalog.yaml", "catalogs:\n  legacy:\n    '@acme/legacy': workspace:~\n")],
    );
    assert!(include
        .duplicates
        .iter()
        .any(|setting| setting == "catalogs"));
    assert!(include.catalogs.get("legacy", "@acme/legacy").is_none());
}

#[test]
fn missing_root_file_yields_no_include() {
    let sources = crate::codebase::rules::source_store_for_files(&[]);
    let resolved =
        resolve_pnpm_include_at_root(Path::new("/repo/does-not-exist"), &sources).unwrap();
    assert!(resolved.is_none());
}
