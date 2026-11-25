use super::*;

#[test]
fn validates_all_lockfile_managers_and_empty_parses() {
    assert!(validate_for_planning(PackageManager::Npm, "   ").is_ok());
    assert_eq!(
        validate_for_planning(
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\nimporters:\n  .: {}\n"
        )
        .unwrap_err()
        .kind,
        "lockfile-pnpm-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Pnpm, "lockfileVersion: [unclosed")
            .unwrap_err()
            .kind,
        "lockfile-pnpm-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Pnpm, "lockfileVersion: '99.0'\n")
            .unwrap_err()
            .kind,
        "lockfile-pnpm-unsupported-schema"
    );
    assert_eq!(
        validate_for_planning(
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\npackages: {}\n---\nlockfileVersion: '9.0'\nimporters: {}\n"
        )
        .unwrap_err()
        .kind,
        "lockfile-pnpm-non-env-document"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Npm, "{")
            .unwrap_err()
            .kind,
        "lockfile-npm-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Npm, "[]")
            .unwrap_err()
            .kind,
        "lockfile-npm-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Npm, "{}")
            .unwrap_err()
            .kind,
        "lockfile-npm-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Yarn, "__metadata:\n  version: [")
            .unwrap_err()
            .kind,
        "lockfile-yarn-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Yarn, "__metadata:\n  version: 6\n")
            .unwrap_err()
            .kind,
        "lockfile-yarn-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Yarn, "not a lockfile")
            .unwrap_err()
            .kind,
        "lockfile-yarn-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Bun, "{")
            .unwrap_err()
            .kind,
        "lockfile-bun-malformed"
    );
    assert_eq!(
        validate_for_planning(PackageManager::Bun, "{\"packages\": {}}")
            .unwrap_err()
            .kind,
        "lockfile-bun-malformed"
    );
    assert!(validate_for_planning(
        PackageManager::Npm,
        "{\"lockfileVersion\":3,\"packages\":{\"node_modules/a\":{\"version\":\"1.0.0\"}}}"
    )
    .is_ok());
    assert!(validate_for_planning(
        PackageManager::Yarn,
        "__metadata:\n  version: 6\n\"a@npm:^1.0.0\":\n  version: 1.0.0\n"
    )
    .is_ok());
    assert!(
        validate_for_planning(PackageManager::Yarn, "a@^1.0.0:\n  version \"1.0.0\"\n").is_ok()
    );
    assert!(validate_for_planning(
        PackageManager::Bun,
        "{\"packages\":{\"a\":[\"a@1.0.0\",{},\"sha512-x\"]}}"
    )
    .is_ok());
}
