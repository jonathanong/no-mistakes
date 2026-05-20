const assert = require("node:assert/strict");
const { readdirSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const root = join(__dirname, "..", "..");
const nativeBinaryPackages = [
  "next-to-fetch",
  "no-mistakes",
  "playwright-ast-coverage",
  "queue-ast-hop",
  "react-traits",
  "server-ast-routes",
];

test("only native CLI npm packages depend on no-mistakes-core", () => {
  const packagesDir = join(root, "packages");
  const offenders = [];

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
    const shouldDepend = nativeBinaryPackages.includes(manifest.name);
    const depends = Boolean(manifest.dependencies?.["no-mistakes-core"]);
    if (shouldDepend) {
      assert.equal(depends, true, `${manifest.name} should depend on no-mistakes-core`);
      assert.equal(manifest.dependencies["no-mistakes-core"], manifest.version);
      continue;
    }
    for (const field of ["dependencies", "devDependencies", "peerDependencies"]) {
      if (manifest[field]?.["no-mistakes-core"]) {
        offenders.push(`${manifest.name}:${field}`);
      }
    }
  }

  assert.deepEqual(offenders, []);
});

test("native npm packages expose direct executable bin targets", () => {
  for (const name of nativeBinaryPackages) {
    const manifest = JSON.parse(readFileSync(join(root, "packages", name, "package.json"), "utf8"));
    assert.deepEqual(manifest.bin, { [name]: `bin/${name}` });

    const placeholder = readFileSync(join(root, "packages", name, "bin", name), "utf8");
    assert.match(placeholder, /Native binary placeholder/);
  }
});
