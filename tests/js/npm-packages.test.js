const assert = require("node:assert/strict");
const { readdirSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const root = join(__dirname, "..", "..");
const nativeBinaryPackages = ["no-mistakes"];

test("only the expected public npm packages remain", () => {
  const packagesDir = join(root, "packages");
  const manifests = [];

  for (const name of readdirSync(packagesDir)) {
    const manifestPath = join(packagesDir, name, "package.json");
    let manifest;
    try {
      manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    } catch (error) {
      if (error.code === "ENOENT") {
        continue;
      }
      throw error;
    }

    if (manifest.private) {
      continue;
    }
    manifests.push(manifest.name);
    for (const field of ["dependencies", "devDependencies", "peerDependencies"]) {
      if (manifest[field]?.["no-mistakes-core"]) {
        assert.fail(`${manifest.name}:${field} must not depend on no-mistakes-core`);
      }
    }
  }

  assert.deepEqual(manifests.sort(), ["eslint-plugin-no-mistakes", "no-mistakes"]);
});

test("native npm packages expose direct executable bin targets", () => {
  for (const name of nativeBinaryPackages) {
    const manifest = JSON.parse(readFileSync(join(root, "packages", name, "package.json"), "utf8"));
    assert.deepEqual(manifest.bin, { [name]: `bin/${name}` });

    const placeholder = readFileSync(join(root, "packages", name, "bin", name), "utf8");
    assert.match(placeholder, /Native binary placeholder/);
  }
});
