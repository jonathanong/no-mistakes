const assert = require("node:assert/strict");
const { execFileSync, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { capture, restore, past, manifestName } = require("./cargo-cache-inputs.cjs");

const fixtures = path.resolve(__dirname, "../../test-cases/cargo-cache-inputs");
function scenario(callback) {
  const directory = fs.mkdtempSync(path.join(tmpdir(), "nm-cargo-cache-inputs-"));
  const workspace = path.join(directory, "workspace");
  const sidecar = path.join(directory, "sidecar");
  fs.cpSync(path.join(fixtures, "workspace"), workspace, { recursive: true });
  const git = (args) =>
    execFileSync("git", ["-C", workspace, ...args], { encoding: "utf8" }).trim();
  git(["init", "--quiet", "--template="]);
  git(["add", "."]);
  const source = path.join(workspace, "src/lib.rs");
  const manifest = path.join(sidecar, manifestName);
  try {
    callback({ directory, workspace, sidecar, source, manifest, git });
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
const old = (file) => fs.utimesSync(file, past, past);
const mtime = (file) => fs.statSync(file).mtimeMs;

test("content matches recover after a fresh checkout touches compiler inputs", () =>
  scenario(({ workspace, sidecar, source, manifest }) => {
    const bytes = fs.readFileSync(source);
    assert.deepEqual(capture(workspace, sidecar), { captured: 3 });
    fs.utimesSync(source, new Date(), new Date());
    const result = restore(workspace, sidecar);
    assert.deepEqual(result, { cached: true, restored: 3, fresh: 0 });
    assert.equal(mtime(source), past.getTime());
    assert.deepEqual(fs.readFileSync(source), bytes);
    const data = JSON.parse(fs.readFileSync(manifest, "utf8"));
    assert.equal(data.version, 1);
    assert.equal(data.workspace, fs.realpathSync(workspace));
    assert.equal(data.files.length, 3);
    assert.match(data.files.find((file) => file.path === "src/lib.rs").sha256, /^[0-9a-f]{64}$/);
    assert.deepEqual(fs.readdirSync(sidecar), [manifestName]);
  }));

test("changed content is fresh even when it inherited an old timestamp", () =>
  scenario(({ workspace, sidecar, source }) => {
    capture(workspace, sidecar);
    fs.copyFileSync(path.join(fixtures, "variants/lib.rs"), source);
    old(source);
    const before = Date.now();
    assert.deepEqual(restore(workspace, sidecar), { cached: true, restored: 2, fresh: 1 });
    assert.ok(mtime(source) >= before - 1);
    assert.deepEqual(
      fs.readFileSync(source),
      fs.readFileSync(path.join(fixtures, "variants/lib.rs")),
    );
  }));

test("deleted inputs disappear and new tracked files stay fresh", () =>
  scenario(({ workspace, sidecar, git }) => {
    capture(workspace, sidecar);
    git(["rm", "--quiet", "-f", "deleted.txt"]);
    const added = path.join(workspace, "src/new.rs");
    fs.copyFileSync(path.join(fixtures, "variants/new.rs"), added);
    git(["add", "src/new.rs"]);
    old(added);
    assert.deepEqual(restore(workspace, sidecar), { cached: true, restored: 2, fresh: 1 });
    assert.ok(mtime(added) > past.getTime());
  }));

test("an absent sidecar is a no-op and untracked files are never traversed", () =>
  scenario(({ workspace, sidecar, source }) => {
    old(source);
    assert.deepEqual(restore(workspace, sidecar), { cached: false, restored: 0, fresh: 0 });
    assert.equal(mtime(source), past.getTime());
    assert.equal(fs.existsSync(sidecar), false);
    const ignored = path.join(workspace, "ignored.txt");
    fs.copyFileSync(path.join(fixtures, "variants/ignored.txt"), ignored);
    old(ignored);
    capture(workspace, sidecar);
    restore(workspace, sidecar);
    assert.equal(mtime(ignored), past.getTime());
  }));

test("Git symlink modes are omitted even when Windows materializes links as text", () =>
  scenario(({ workspace, sidecar, manifest, git }) => {
    const link = path.join(workspace, "materialized-link");
    fs.copyFileSync(path.join(fixtures, "variants/ignored.txt"), link);
    const hash = git(["hash-object", "-w", "materialized-link"]);
    git(["update-index", "--add", "--cacheinfo", `120000,${hash},materialized-link`]);
    old(link);
    assert.deepEqual(capture(workspace, sidecar), { captured: 3 });
    const data = JSON.parse(fs.readFileSync(manifest, "utf8"));
    assert.equal(
      data.files.some((row) => row.path === "materialized-link"),
      false,
    );
    restore(workspace, sidecar);
    assert.equal(mtime(link), past.getTime());
  }));

test("malformed, unsafe and mismatched manifests fail before changing any input", () => {
  const expected = {
    malformed: { name: "SyntaxError", message: "Unexpected end of JSON input" },
    traversal: { message: 'Unsafe input path: "../outside"' },
    absolute: { message: 'Unsafe input path: "/outside"' },
    windows: { message: 'Unsafe input path: "C:\\\\outside"' },
    "invalid-hash": { message: "Invalid input hash: src/lib.rs" },
    "wrong-workspace": { message: "Input manifest workspace differs from this checkout" },
    duplicate: { message: "Duplicate input manifest path: src/lib.rs" },
  };
  for (const [name, error] of Object.entries(expected))
    scenario(({ workspace, sidecar, source, manifest }) => {
      fs.mkdirSync(sidecar);
      fs.copyFileSync(path.join(fixtures, "manifests", `${name}.json`), manifest);
      fs.utimesSync(source, new Date(), new Date());
      const before = mtime(source);
      assert.throws(() => restore(workspace, sidecar), error, name);
      assert.equal(mtime(source), before, name);
    });
});

test("a tracked regular file replaced by a symlink cannot publish or restore inputs", () =>
  scenario(({ directory, workspace, sidecar, source, manifest }) => {
    capture(workspace, sidecar);
    const before = fs.readFileSync(manifest);
    const outside = path.join(directory, "outside.rs");
    fs.copyFileSync(path.join(fixtures, "variants/lib.rs"), outside);
    old(outside);
    fs.rmSync(source);
    fs.symlinkSync(outside, source);
    assert.throws(() => capture(workspace, sidecar), /symlink/);
    assert.throws(() => restore(workspace, sidecar), /symlink/);
    assert.equal(mtime(outside), past.getTime());
    assert.deepEqual(fs.readFileSync(manifest), before);
    assert.deepEqual(fs.readdirSync(sidecar), [manifestName]);
  }));

test("symlink parents, symlink sidecars and nested workspaces are rejected", () =>
  scenario(({ directory, workspace, sidecar }) => {
    capture(workspace, sidecar);
    assert.throws(() => restore(path.join(workspace, "src"), sidecar), /top-level/);
    const alias = path.join(directory, "alias");
    fs.symlinkSync(workspace, alias, "junction");
    assert.throws(() => restore(alias, sidecar), /symlink/);
    const sidecarAlias = path.join(directory, "sidecar-alias");
    fs.symlinkSync(sidecar, sidecarAlias, "junction");
    assert.throws(() => restore(workspace, sidecarAlias), /symlink/);
    fs.renameSync(path.join(workspace, "src"), path.join(directory, "outside-src"));
    fs.symlinkSync(path.join(directory, "outside-src"), path.join(workspace, "src"), "junction");
    assert.throws(() => restore(workspace, sidecar), /symlink/);
  }));

test("CLI capture and restore require an explicit valid mode and Git workspace", () =>
  scenario(({ workspace, sidecar }) => {
    const script = path.join(__dirname, "cargo-cache-inputs.cjs");
    for (const mode of ["capture", "restore"]) {
      const result = spawnSync(process.execPath, [script, mode, workspace, sidecar], {
        encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      assert.ok(JSON.parse(result.stdout));
    }
    for (const args of [
      [],
      ["unknown", workspace, sidecar],
      ["restore", path.join(workspace, "src"), sidecar],
    ]) {
      const result = spawnSync(process.execPath, [script, ...args], { encoding: "utf8" });
      assert.equal(result.status, 1);
    }
  }));
