const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const {
  cargoReleaseDirectory,
  cargoTargetDirectory,
  runCargoMetadata,
} = require("./native-artifact-paths");

const repositoryRoot = "/repo/no-mistakes";

function metadata(targetDirectory) {
  return JSON.stringify({ target_directory: targetDirectory });
}

test("asks Cargo for metadata from the staging root with its build environment", () => {
  const env = { CARGO_TARGET_DIR: "/external/rust-target" };
  assert.equal(
    runCargoMetadata({
      root: repositoryRoot,
      env,
      spawn: (command, args, options) => {
        assert.equal(command, "cargo");
        assert.deepEqual(args, ["metadata", "--format-version", "1", "--no-deps"]);
        assert.deepEqual(options, { cwd: repositoryRoot, encoding: "utf8", env });
        return { status: 0, stdout: metadata("/external/rust-target"), stderr: "" };
      },
    }),
    metadata("/external/rust-target"),
  );
});

test("uses Cargo metadata's default worktree-local target directory", () => {
  assert.equal(
    cargoTargetDirectory({
      root: repositoryRoot,
      env: {},
      runCargoMetadata: () => metadata("/repo/no-mistakes/target"),
    }),
    "/repo/no-mistakes/target",
  );
  assert.equal(
    cargoReleaseDirectory({
      root: repositoryRoot,
      env: {},
      runCargoMetadata: () => metadata("/repo/no-mistakes/target"),
    }),
    join("/repo/no-mistakes/target", "release"),
  );
});

test("uses a Cargo configuration target directory for local native staging", () => {
  assert.equal(
    cargoTargetDirectory({
      root: repositoryRoot,
      env: {},
      runCargoMetadata: () => metadata("/configured/cargo-target"),
    }),
    "/configured/cargo-target",
  );
});

test("passes CARGO_TARGET_DIR to Cargo metadata instead of resolving it independently", () => {
  const env = { CARGO_TARGET_DIR: "/external/rust-target" };
  assert.equal(
    cargoReleaseDirectory({
      root: repositoryRoot,
      env,
      runCargoMetadata: (options) => {
        assert.equal(options.root, repositoryRoot);
        assert.equal(options.env, env);
        return metadata("/external/rust-target");
      },
    }),
    join("/external/rust-target", "release"),
  );
});

test("keeps Cargo's metadata result for a relative target setting", () => {
  assert.equal(
    cargoTargetDirectory({
      root: repositoryRoot,
      env: { CARGO_TARGET_DIR: "relative-target" },
      runCargoMetadata: () => metadata("/repo/no-mistakes/relative-target"),
    }),
    "/repo/no-mistakes/relative-target",
  );
});

test("rejects malformed Cargo metadata", () => {
  assert.throws(
    () =>
      cargoTargetDirectory({
        root: repositoryRoot,
        runCargoMetadata: () => "not json",
      }),
    /Cargo metadata did not return valid JSON/,
  );
});

test("propagates a failed Cargo metadata invocation", () => {
  assert.throws(
    () =>
      cargoTargetDirectory({
        root: repositoryRoot,
        runCargoMetadata: () => {
          throw new Error("cargo metadata failed (1): bad configuration");
        },
      }),
    /cargo metadata failed \(1\): bad configuration/,
  );
});

test("reports a failed Cargo metadata command", () => {
  assert.throws(
    () =>
      runCargoMetadata({
        root: repositoryRoot,
        spawn: () => ({ status: 1, stderr: "invalid target directory" }),
      }),
    /cargo metadata failed: invalid target directory/,
  );
});
