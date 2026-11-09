const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const { fixtureRelativePath, normalizeFixtureReport } = require("./fixture-relative-path.js");

const cases = JSON.parse(
  readFileSync(
    join(__dirname, "../../../test-cases/integration/napi-fixture-paths/fixture/paths.json"),
    "utf8",
  ),
);

test("fixture-relative path normalizes saved cross-platform cases", () => {
  for (const { root, file, expected } of cases.accepted) {
    assert.equal(fixtureRelativePath(file, root), expected);
  }
  for (const { root, file } of cases.rejected) {
    assert.throws(() => fixtureRelativePath(file, root), /outside or on a different root/u);
  }
});

test("fixture report normalization changes only exact fixture-root aliases", () => {
  const { root, aliases, changedError, forwardSlashAlias, similarPrefix, outside } = cases.reports;
  const normalized = aliases.map(({ input, expected }) => {
    assert.deepEqual(normalizeFixtureReport(input, root), expected);
    return normalizeFixtureReport(input, root);
  });
  assert.deepEqual(normalized[0], normalized[1]);
  assert.deepEqual(
    normalizeFixtureReport(forwardSlashAlias.input, root),
    forwardSlashAlias.expected,
  );
  assert.deepEqual(normalizeFixtureReport(forwardSlashAlias.input, root), normalized[0]);
  assert.deepEqual(normalizeFixtureReport(changedError.input, root), changedError.expected);
  assert.notDeepEqual(normalizeFixtureReport(changedError.input, root), normalized[0]);
  assert.deepEqual(normalizeFixtureReport(similarPrefix.input, root), similarPrefix.expected);
  assert.deepEqual(normalizeFixtureReport(outside.input, root), outside.expected);
  const posix = cases.posixReport;
  assert.deepEqual(normalizeFixtureReport(posix.input, posix.root), posix.expected);
});
