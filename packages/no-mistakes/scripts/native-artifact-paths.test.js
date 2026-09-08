const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const { cargoReleaseDirectory, cargoTargetDirectory } = require("./native-artifact-paths");

const repositoryRoot = "/repo/no-mistakes";

test("uses Cargo's default worktree-local target directory", () => {
  assert.equal(
    cargoTargetDirectory({ root: repositoryRoot, env: {} }),
    join(repositoryRoot, "target"),
  );
  assert.equal(
    cargoReleaseDirectory({ root: repositoryRoot, env: {} }),
    join(repositoryRoot, "target", "release"),
  );
});

test("uses CARGO_TARGET_DIR for local native staging", () => {
  assert.equal(
    cargoTargetDirectory({
      root: repositoryRoot,
      env: { CARGO_TARGET_DIR: "/external/rust-target" },
    }),
    "/external/rust-target",
  );
  assert.equal(
    cargoReleaseDirectory({
      root: repositoryRoot,
      env: { CARGO_TARGET_DIR: "/external/rust-target" },
    }),
    "/external/rust-target/release",
  );
});
