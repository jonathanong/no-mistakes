use super::*;

#[test]
fn validates_all_lockfile_managers_and_zero_dependency_lockfiles() {
    let invalid_cases = [
        (
            PackageManager::Pnpm,
            "lockfileVersion: [unclosed",
            "lockfile-pnpm-malformed",
        ),
        (
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\n",
            "lockfile-pnpm-unsupported-schema",
        ),
        (
            PackageManager::Pnpm,
            "lockfileVersion: '99.0'\n",
            "lockfile-pnpm-unsupported-schema",
        ),
        (
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\npackages: {}\n---\nlockfileVersion: '9.0'\nimporters: {}\n",
            "lockfile-pnpm-non-env-document",
        ),
        (
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\npackages:\n  a@1.0.0:\n    resolution: {integrity: sha512-x}\n  bad@1.0.0: unexpected\n",
            "lockfile-pnpm-unsupported-schema",
        ),
        (PackageManager::Npm, "{", "lockfile-npm-malformed"),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":3}",
            "lockfile-npm-malformed",
        ),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":3,\"packages\":{\"node_modules/a\":{\"version\":\"1.0.0\"},\"node_modules/b\":\"bad\"}}",
            "lockfile-npm-malformed",
        ),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":4,\"packages\":{}}",
            "lockfile-npm-unsupported-version",
        ),
        (
            PackageManager::Yarn,
            "__metadata:\n  version: [",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "# __metadata:\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "# random comment\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "not a lockfile",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "a@^1.0.0:\n  version \"1.0.0\"\nb@^1.0.0:\n  unexpected \"x\"\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "a@^1.0.0:\n  version \"1.0.0\"\nnot-a-header\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "  version \"1.0.0\"\na@^1.0.0:\n  version \"1.0.0\"\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "\"\":\n  version \"1.0.0\"\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "__metadata:\n  version: 6\n\"a@npm:^1.0.0\":\n  version: 1.0.0\n\"b@npm:^1.0.0\": \"bad\"\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "__metadata:\n  version: 6\n\"a@npm:^1.0.0\":\n  version: 1.0.0\n123:\n  version: 1.0.0\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "__metadata: \"bad\"\n\"a@npm:^1.0.0\":\n  version: 1.0.0\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "__metadata:\n  version: 6\n\"a@npm:^1.0.0\":\n  version: 1.0.0\n\"b@npm:^1.0.0\":\n  languageName: node\n",
            "lockfile-yarn-malformed",
        ),
        (
            PackageManager::Yarn,
            "- __metadata:\n    version: 6\n",
            "lockfile-yarn-malformed",
        ),
        (PackageManager::Bun, "{", "lockfile-bun-malformed"),
        (PackageManager::Bun, "{}", "lockfile-bun-malformed"),
        (
            PackageManager::Bun,
            "{\"packages\":{\"a\":[\"a@1.0.0\",{},\"sha512-x\"],\"b\":\"bad\"}}",
            "lockfile-bun-malformed",
        ),
        (
            PackageManager::Bun,
            "{\"packages\":{\"a\":[\"a@1.0.0\",{},\"sha512-x\"],\"b\":[]}}",
            "lockfile-bun-malformed",
        ),
        (
            PackageManager::Bun,
            "{\"packages\":{\"a\":[\"a@1.0.0\",{},\"sha512-x\"],\"\":[\"b@1.0.0\",{}]}}",
            "lockfile-bun-malformed",
        ),
    ];
    for (manager, content, expected_kind) in invalid_cases {
        assert_eq!(
            validate_for_planning(manager, content).unwrap_err().kind,
            expected_kind,
            "{manager:?}: {content:?}"
        );
    }

    let valid_cases = [
        (
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\nimporters:\n  .: {}\n",
        ),
        (
            PackageManager::Pnpm,
            "lockfileVersion: '9.0'\npackages: {}\n",
        ),
        (PackageManager::Npm, "   "),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":3,\"packages\":{}}",
        ),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":3,\"packages\":{\"\":{\"name\":\"app\",\"version\":\"1.0.0\"}}}",
        ),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":1,\"dependencies\":{}}",
        ),
        (
            PackageManager::Npm,
            "{\"lockfileVersion\":3,\"packages\":{\"node_modules/a\":{\"version\":\"1.0.0\"}}}",
        ),
        (PackageManager::Yarn, "__metadata:\n  version: 6\n"),
        (
            PackageManager::Yarn,
            "# THIS IS AN AUTOGENERATED FILE.\n# yarn lockfile v1\n",
        ),
        (
            PackageManager::Yarn,
            "__metadata:\n  version: 6\n\"a@npm:^1.0.0\":\n  version: 1.0.0\n",
        ),
        (PackageManager::Yarn, "a@^1.0.0:\n  version \"1.0.0\"\n"),
        (PackageManager::Bun, "{\"packages\": {}}"),
        (
            PackageManager::Bun,
            "{\"lockfileVersion\":0,\"workspaces\":{\"\":{}},\"packages\":{}}",
        ),
        (
            PackageManager::Bun,
            "{\"packages\":{\"a\":[\"a@1.0.0\",{},\"sha512-x\"]}}",
        ),
    ];
    for (manager, content) in valid_cases {
        assert!(
            validate_for_planning(manager, content).is_ok(),
            "{manager:?}: {content:?}"
        );
    }
}
