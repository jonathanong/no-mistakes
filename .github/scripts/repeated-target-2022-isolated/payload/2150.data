const assert = require("node:assert/strict");
const { basename, dirname, normalize } = require("node:path");
const test = globalThis.test || require("node:test").test;

const { localNativeArtifacts } = require("./local-stage-native");

test("local staging consumes the Cargo metadata release-directory resolver", () => {
  const env = { CARGO_TARGET_DIR: "/external/rust-target" };
  const artifacts = localNativeArtifacts({
    root: "/repo/no-mistakes",
    env,
    platform: "linux",
    addonExists: () => false,
    resolveReleaseDirectory: (options) => {
      assert.deepEqual(options, { root: "/repo/no-mistakes", env });
      return "/configured/cargo-target/release";
    },
  });

  assert.equal(basename(artifacts.binary), "no-mistakes");
  assert.equal(basename(artifacts.addon), "libno_mistakes.so");
  assert.equal(dirname(artifacts.binary), normalize("/configured/cargo-target/release"));
  assert.equal(dirname(artifacts.addon), normalize("/configured/cargo-target/release"));
});
