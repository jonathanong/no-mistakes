const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { run } = require("./lychee-gateway-fixture.cjs");
const saved = path.resolve(__dirname, "../../test-cases/ci/lychee-gateway");
const read = (name) => readFileSync(path.join(saved, name), "utf8");

for (const report of ["gateway.md", "github-503.md", "github-501.md", "github-http.md"]) {
  test(
    `exhausted ${report} retries warn after two complete checks`,
    { skip: process.platform === "win32" },
    () => {
      const result = run(report, {
        LYCHEE_ARGS: "--no-progress README.md 'docs/**/*.md'",
        LYCHEE_GATEWAY_WAIT: "0,0",
        LYCHEE_FAKE_FAILURE: path.join(saved, report),
      });
      assert.equal(result.status, 0);
      assert.equal(result.count, "2");
      assert.equal(result.report, read(report));
      assert.match(result.output, /inconclusiveGitHubLinks/);
      assert.match(result.output, /::warning::/);
      assert.equal(result.argv.trim().split("\n").length, 2);
      assert.match(result.argv, /README\.md docs\/\*\*\/\*\.md/);
    },
  );
}

for (const report of [
  "github-404.md",
  "github-403.md",
  "github-mixed.md",
  "malformed.md",
  "mismatched.md",
  "timeout.md",
  "missing-timeout.md",
  "truncated.md",
  "truncated-after-url.md",
  "duplicate-summary.md",
]) {
  test(`${report} cannot become a warning`, () => {
    const result = run(report, { LYCHEE_ARGS: "--no-progress README.md" });
    assert.equal(result.status, 2);
    assert.equal(result.count, "");
    assert.doesNotMatch(result.output, /::warning::/);
  });
}

for (const report of ["external-503.md", "spoof-host.md", "userinfo-host.md"]) {
  test(`${report} remains fatal after retries`, () => {
    const result = run(report, {
      LYCHEE_ARGS: "--no-progress README.md",
      LYCHEE_GATEWAY_WAIT: "0,0",
      LYCHEE_FAKE_FAILURE: path.join(saved, report),
    });
    assert.notEqual(result.status, 0);
    assert.equal(result.count, "2");
    assert.doesNotMatch(result.output, /::warning::/);
  });
}

for (const env of [{ LYCHEE_FAKE_EXIT: "1" }, { LYCHEE_FAKE_NO_REPORT: "1" }]) {
  test(
    `checker failure ${JSON.stringify(env)} cannot reuse a gateway report`,
    { skip: process.platform === "win32" },
    () => {
      const result = run("gateway.md", { LYCHEE_ARGS: "--no-progress README.md", ...env });
      assert.notEqual(result.status, 0);
      assert.equal(result.count, "1");
      assert.doesNotMatch(result.output, /::warning::/);
      if (env.LYCHEE_FAKE_NO_REPORT) assert.equal(result.report, "");
    },
  );
}

test(
  "a later external-host failure cannot inherit the GitHub warning exception",
  { skip: process.platform === "win32" },
  () => {
    const result = run("gateway.md", {
      LYCHEE_ARGS: "--no-progress README.md",
      LYCHEE_GATEWAY_WAIT: "0,0",
      LYCHEE_FAKE_FAILURE: path.join(saved, "external-503.md"),
    });
    assert.notEqual(result.status, 0);
    assert.equal(result.count, "2");
    assert.doesNotMatch(result.output, /::warning::/);
  },
);
