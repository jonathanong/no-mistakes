const assert = require("node:assert/strict");
const { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const { PACKAGE_NAMES, syncNativePackageVersions } = require("./sync-native-package-versions");

test("release version sync pins every optional native package to the main package version", () => {
  const root = mkdtempSync(join(tmpdir(), "no-mistakes-version-sync-"));
  try {
    for (const name of [...PACKAGE_NAMES, "no-mistakes"]) {
      mkdirSync(join(root, "packages", name), { recursive: true });
      writeFileSync(
        join(root, "packages", name, "package.json"),
        JSON.stringify(
          name === "no-mistakes" ? { optionalDependencies: {} } : { version: "0.0.0" },
        ),
      );
    }
    syncNativePackageVersions(root, "1.2.3");
    for (const name of PACKAGE_NAMES) {
      const value = JSON.parse(readFileSync(join(root, "packages", name, "package.json"), "utf8"));
      assert.equal(value.version, "1.2.3");
    }
    const main = JSON.parse(
      readFileSync(join(root, "packages", "no-mistakes", "package.json"), "utf8"),
    );
    assert.deepEqual(Object.values(main.optionalDependencies), Array(5).fill("1.2.3"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
