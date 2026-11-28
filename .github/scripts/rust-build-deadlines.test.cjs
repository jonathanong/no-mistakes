const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const evidence = require("./fixtures/rust-build-deadlines.json");
const workflows = Object.fromEntries(
  ["ci", "release", "benchmarks"].map((name) => [
    name,
    readFileSync(path.join(__dirname, `../workflows/${name}.yml`), "utf8"),
  ]),
);
function seconds(value) {
  if (/^\d+$/.test(value)) return Number(value) * 60;
  const branches = value.match(/&& (\d+) \|\| (\d+)/);
  assert.ok(branches, `unsupported deadline: ${value}`);
  return Math.max(Number(branches[1]), Number(branches[2])) * 60;
}
const budget = (body) => seconds(body.match(/^\s+timeout-minutes: (.+)$/m)[1]);
const step = (body, name) => body.split(`      - name: ${name}\n`)[1].split(/\n      - /)[0];
for (const sample of evidence.censored) {
  test(`cold timeout reserve: ${sample.workflow}/${sample.job}`, () => {
    const body = workflowJob(workflows[sample.workflow], sample.job);
    const stepSeconds = budget(step(body, sample.step));
    assert.ok(stepSeconds >= sample.seconds * 1.25, `retain cold reserve for run ${sample.run}`);
    assert.ok(budget(body) >= stepSeconds + 300, "setup and artifact reserve must fit");
  });
}
test("measured CI exceptions remain finite and below the sum of their bounded steps", () => {
  for (const name of [
    "quality-rust",
    "packages",
    "linux-runtime",
    "native-api-tests",
    "native-api-tests-macos",
    "native-test-build",
    "native-rust-tests-macos",
    "standalone-build",
    "rust-coverage-build",
    "rust-coverage-build-integration",
    "docs-tests",
    "bench",
  ]) {
    const body = workflowJob(name === "bench" ? workflows.benchmarks : workflows.ci, name);
    const jobSeconds = budget(body);
    const steps = body.split("    steps:\n")[1];
    const limits = [...steps.matchAll(/^        timeout-minutes: (.+)$/gm)].map((m) =>
      seconds(m[1]),
    );
    assert.ok(limits.length, name);
    assert.ok(Math.max(...limits) < jobSeconds, `${name}: steps need job headroom`);
    assert.ok(
      jobSeconds < limits.reduce((sum, value) => sum + value, 0),
      `${name}: preserve the step-sum bound`,
    );
    assert.ok(jobSeconds <= 90 * 60, `${name}: retain a finite build bound`);
  }
});
test("the generic repository cap is preserved with local measured exceptions", () => {
  const config = readFileSync(path.join(__dirname, "../../.no-mistakes.yml"), "utf8");
  assert.match(config, /rule: github-actions-job-timeouts[\s\S]*?maxMinutes: 30/);
  assert.match(config, /rule: github-actions-step-timeouts[\s\S]*?maxMinutes: 30/);
  const lines = [...workflows.ci.split("\n"), ...workflows.benchmarks.split("\n")];
  for (let i = 0; i < lines.length; i++) {
    if (!lines[i].includes("no-mistakes-disable-next-line github-actions-")) continue;
    assert.match(lines[i], /#3514.*rust-build-deadlines.test.cjs/);
    assert.match(lines[i + 1], /^  [a-z][a-z-]*:|^      - name:/);
  }
});

test("each quality matrix branch retains job headroom", () => {
  const body = workflowJob(workflows.ci, "quality-rust");
  const resolve = (text, name) => {
    const value = text.match(/^\s+timeout-minutes: (.+)$/m)[1];
    const condition = value.match(/matrix\.name == '([^']+)' && (\d+) \|\| (\d+)/);
    assert.ok(condition, value);
    return Number(name === condition[1] ? condition[2] : condition[3]) * 60;
  };
  for (const name of ["PostgreSQL source identities", "all-feature Clippy", "formatting"]) {
    assert.ok(
      resolve(body, name) >= resolve(step(body, "Run isolated Rust quality check"), name) + 300,
    );
  }
});
