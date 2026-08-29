const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { mkdtempSync, readdirSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join, posix } = require("node:path");

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
test("npm pack includes every transitive local require() from a published entry point", () => {
  const packageDir = join(root, "packages", "no-mistakes");
  const manifest = JSON.parse(readFileSync(join(packageDir, "package.json"), "utf8"));
  const entryPoints = ["index.js", "planning.js", "workflow-topology-index.js"];
  const requirePattern = /require\("\.\/([\w./-]+)"\)/g;
  const npmCache = mkdtempSync(join(tmpdir(), "no-mistakes-npm-pack-"));
  let packed;
  try {
    const npmArguments = ["pack", "--dry-run", "--ignore-scripts", "--json"];
    const command = process.platform === "win32" ? process.env.ComSpec || "cmd.exe" : "npm";
    const commandArguments =
      process.platform === "win32" ? ["/d", "/s", "/c", "npm", ...npmArguments] : npmArguments;
    packed = JSON.parse(
      execFileSync(command, commandArguments, {
        cwd: packageDir,
        encoding: "utf8",
        env: { ...process.env, NPM_CONFIG_CACHE: npmCache },
      }),
    );
  } finally {
    rmSync(npmCache, { recursive: true, force: true });
  }
  const packedPaths = new Set(packed[0].files.map((file) => file.path));

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

  const checked = new Set();
  const resolveLocalRequire = (entry, required) => {
    const requestedPath = posix.join(posix.dirname(entry), required);
    return requestedPath.startsWith("bin/") ? requestedPath : `${requestedPath}.js`;
  };
  assert.equal(resolveLocalRequire("sub/a.js", "b"), "sub/b.js");
  const checkEntry = (entry) => {
    if (checked.has(entry)) return;
    checked.add(entry);
    const source = readFileSync(join(packageDir, entry), "utf8");
    for (const match of source.matchAll(requirePattern)) {
      const required = match[1];
      // `./bin/no-mistakes.node` is the native addon, covered by `bin/`
      // regardless of extension; every other local require resolves to a
      // sibling `.js` file the same way Node's CJS resolver would.
      const relativePath = resolveLocalRequire(entry, required);
      assert.ok(
        isCovered(relativePath),
        `${entry} requires "./${required}" but ${relativePath} is not covered by package.json's files list`,
      );
      assert.ok(
        packedPaths.has(relativePath),
        `${entry} requires "./${required}" but npm pack does not include ${relativePath}`,
      );
      if (!relativePath.startsWith("bin/")) checkEntry(relativePath);
    }
  };

  entryPoints.forEach(checkEntry);
});
