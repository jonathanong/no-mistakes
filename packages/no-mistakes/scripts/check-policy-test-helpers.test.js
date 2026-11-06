const assert = require("node:assert/strict");
const {
  normalizeCheckFailurePaths,
  normalizeVerbatimWindowsPath,
} = require("./check-policy-test-helpers");
const test = globalThis.test || require("node:test").test;
const slash = String.fromCharCode(92);
const local = ["D:", "repo", "src", "broken.ts"].join(slash);
const verbatim = slash.repeat(2) + "?" + slash + local;

test("check failure comparisons normalize only the same verbatim filename", () => {
  const unrelated = slash.repeat(2) + "?" + slash + "D:" + slash + "other.ts";
  const report = {
    rules: [{ message: verbatim }],
    incomplete: [
      {
        consumer: "forbidden-lint-suppressions",
        file: verbatim,
        property: "source",
        message: `parsing ${verbatim}: Unexpected token at 46; ${unrelated}`,
      },
      { consumer: "queue", file: local, message: "" },
      { consumer: "filesystem", message: "input unavailable" },
    ],
  };
  const before = structuredClone(report);
  assert.deepEqual(normalizeCheckFailurePaths(report), {
    rules: report.rules,
    incomplete: [
      {
        consumer: "forbidden-lint-suppressions",
        file: local,
        property: "source",
        message: `parsing ${local}: Unexpected token at 46; ${unrelated}`,
      },
      { consumer: "queue", file: local, message: "" },
      { consumer: "filesystem", message: "input unavailable" },
    ],
  });
  assert.deepEqual(report, before);
});

test("healthy reports retain identity and drive and UNC paths retain their meaning", () => {
  const report = { rules: [] };
  assert.equal(normalizeCheckFailurePaths(report), report);
  assert.equal(normalizeVerbatimWindowsPath(verbatim), local);
  const unc = slash.repeat(2) + "server" + slash + "share";
  assert.equal(
    normalizeVerbatimWindowsPath(
      slash.repeat(2) + "?" + slash + "UNC" + slash + "server" + slash + "share",
    ),
    unc,
  );
  for (const path of [local, unc, "/repo/src/broken.ts", "", undefined])
    assert.equal(normalizeVerbatimWindowsPath(path), path);
});
