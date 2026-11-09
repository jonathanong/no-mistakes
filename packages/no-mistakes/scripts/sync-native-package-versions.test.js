const assert = require("node:assert/strict");
const { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const {
  PACKAGE_NAMES,
  PLUGIN_MANIFESTS,
  syncNativePackageVersions,
} = require("./sync-native-package-versions");

function writeJson(path, value) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify(value));
}

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

test("release version sync pins native packages and agent plugins to the main package version", () => {
  const root = mkdtempSync(join(tmpdir(), "no-mistakes-version-sync-"));
  try {
    for (const name of PACKAGE_NAMES) {
      writeJson(join(root, "packages", name, "package.json"), { version: "0.0.0" });
    }
    writeJson(join(root, "packages", "no-mistakes", "package.json"), {
      optionalDependencies: {},
    });
    const plugin = (version) => `{\n  "version": "${version}",\n  "tags": ["a", "b"]\n}\n`;
    for (const file of PLUGIN_MANIFESTS) {
      mkdirSync(dirname(join(root, file)), { recursive: true });
      writeFileSync(join(root, file), plugin("0.0.0"));
    }
    syncNativePackageVersions(root, "1.2.3");
    for (const name of PACKAGE_NAMES) {
      assert.equal(readJson(join(root, "packages", name, "package.json")).version, "1.2.3");
    }
    const main = readJson(join(root, "packages", "no-mistakes", "package.json"));
    assert.deepEqual(Object.values(main.optionalDependencies), Array(4).fill("1.2.3"));
    for (const file of PLUGIN_MANIFESTS) {
      assert.equal(readFileSync(join(root, file), "utf8"), plugin("1.2.3"));
    }
    syncNativePackageVersions(root, "1.2.3");
    assert.equal(readFileSync(join(root, PLUGIN_MANIFESTS[0]), "utf8"), plugin("1.2.3"));
    writeJson(join(root, PLUGIN_MANIFESTS[0]), { name: "no-mistakes" });
    assert.throws(() => syncNativePackageVersions(root, "1.2.4"), /has no "version" field/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
