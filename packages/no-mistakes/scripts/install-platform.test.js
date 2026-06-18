const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { supportedGlibc } = require("./install/platform.js");

test("supportedGlibc parses valid versions correctly", () => {
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "2.35" } }),
    }),
    true
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "2.36" } }),
    }),
    true
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "3.0" } }),
    }),
    true
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "2.34" } }),
    }),
    false
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "1.99" } }),
    }),
    false
  );
});

test("supportedGlibc uses glibcVersionCompiler when runtime is unavailable", () => {
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionCompiler: "2.35" } }),
    }),
    true
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionCompiler: "2.34" } }),
    }),
    false
  );
});

test("supportedGlibc handles missing or invalid reports", () => {
  assert.equal(supportedGlibc({}), false);
  assert.equal(
    supportedGlibc({
      getReport: () => ({}),
    }),
    false
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: {} }),
    }),
    false
  );
});

test("supportedGlibc handles invalid version formats", () => {
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "invalid" } }),
    }),
    false
  );
  assert.equal(
    supportedGlibc({
      getReport: () => ({ header: { glibcVersionRuntime: "2.invalid" } }),
    }),
    false
  );
});
