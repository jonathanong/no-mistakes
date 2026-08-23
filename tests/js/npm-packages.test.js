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

// A `require("./sibling")` in a published entry point that isn't covered by
// `package.json`'s `files` list is absent from the published tarball —
// `require("no-mistakes")` throws MODULE_NOT_FOUND for every consumer, not
// just the feature that needed the missing file. Caught once already
// (`workflow-topology-index.js`); this generalizes the check so the next
// sibling module can't repeat it.
test("every local require() from a published entry point is covered by package.json's files list", () => {
  const packageDir = join(root, "packages", "no-mistakes");
  const manifest = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
  const entryPoints = ["index.js", "planning.js"];
  const requirePattern = /require\("\.\/([\w./-]+)"\)/g;

  const isCovered = (relativePath) =>
    manifest.files.some((pattern) => {
      if (pattern.endsWith("/")) return relativePath.startsWith(pattern);
      if (pattern.includes("*")) {
        const escaped = pattern
          .split("*")
          .map((segment) => segment.replace(/[.+?^${}()|[\]\\]/g, "\\$&"))
          .join(".*");
        return new RegExp(`^${escaped}$`).test(relativePath);
      }
      return relativePath === pattern;
    });

  for (const entry of entryPoints) {
    const source = readFileSync(join(packageDir, entry), "utf8");
    for (const match of source.matchAll(requirePattern)) {
      const required = match[1];
      // `./bin/no-mistakes.node` is the native addon, covered by `bin/`
      // regardless of extension; every other local require resolves to a
      // sibling `.js` file the same way Node's CJS resolver would.
      const relativePath = required.startsWith("bin/") ? required : `${required}.js`;
      assert.ok(
        isCovered(relativePath),
        `${entry} requires "./${required}" but ${relativePath} is not covered by package.json's files list`,
      );
    }
  }
});
