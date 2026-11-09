const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const { fixtureRelativePath } = require("./fixture-relative-path.js");

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
