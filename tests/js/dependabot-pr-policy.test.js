const assert = require("node:assert/strict");
const { join } = require("node:path");

const { isFixtureOnlyChange, shouldCloseFixtureOnlyChange } = require(
  join(__dirname, "..", "..", ".github", "scripts", "dependabot-pr-policy.cjs"),
);

test("recognizes fixture-only Dependabot changes", () => {
  assert.equal(isFixtureOnlyChange(["fixtures/example/package.json"]), true);
  assert.equal(isFixtureOnlyChange(["test-cases/example/Cargo.toml"]), true);
  assert.equal(
    isFixtureOnlyChange(["fixtures/example/package.json", "test-cases/example/pnpm-lock.yaml"]),
    true,
  );
});

test("keeps non-fixture Dependabot changes eligible for auto-merge", () => {
  assert.equal(isFixtureOnlyChange([]), false);
  assert.equal(isFixtureOnlyChange(["package.json"]), false);
  assert.equal(isFixtureOnlyChange(["fixtures-old/package.json"]), false);
  assert.equal(isFixtureOnlyChange(["fixtures/example/package.json", "package.json"]), false);
});

test("keeps fixture-only changes open when GitHub truncates the file list", () => {
  const returnedPaths = ["fixtures/example/package.json"];

  assert.equal(shouldCloseFixtureOnlyChange(returnedPaths, 2), false);
  assert.equal(shouldCloseFixtureOnlyChange(returnedPaths, 1), true);
});
