const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { isAbsolute, join, relative, resolve, sep } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = join(__dirname, "../../../test-cases/rules/test-no-live-repository-scan/fixture");
const nodeRoot = join(
  __dirname,
  "../../../test-cases/rules/test-no-live-repository-scan/node-loaders",
);
const rule = "test-no-live-repository-scan";

function normalizedFinding(finding, root) {
  const file = (isAbsolute(finding.file) ? relative(root, finding.file) : finding.file)
    .split(sep)
    .join("/");
  assert.equal(isAbsolute(file), false, file);
  assert.equal(file.startsWith("../"), false, file);
  const sourceLine = readFileSync(resolve(root, file), "utf8")
    .split(/\r?\n/u)
    .at(finding.line - 1)
    ?.trim();
  assert.ok(sourceLine, `${file}:${finding.line} has no fixture source line`);
  return { ...finding, file, sourceLine };
}

function snapshotReport(report, root) {
  const findings = (items) =>
    items
      .filter((finding) => finding.rule === rule)
      .map((finding) => normalizedFinding(finding, root))
      .sort((left, right) => JSON.stringify(left).localeCompare(JSON.stringify(right), "en"));
  return {
    rules: findings(report.rules),
    advisories: findings(report.advisories),
  };
}

function withoutPoolWarning(report) {
  const { jobsPoolWarning: _jobsPoolWarning, ...stable } = report;
  return stable;
}

async function assertParity(cases, snapshotPath) {
  const cjs = require("../index.js");
  const esm = await import("../index.mjs");
  const snapshots = {};

  for (const [label, root, config] of cases) {
    let expected;
    for (const jobs of [1, 2]) {
      const options = { root, config: join(root, config), jobs };
      const pending = cjs.check(options);
      assert.equal(typeof pending.then, "function");
      const report = await pending;
      assert.deepEqual(withoutPoolWarning(await esm.check(options)), withoutPoolWarning(report));
      const request = { ...options, reports: [{ type: "check" }] };
      const prepared = await cjs.analyzeProject(request);
      assert.deepEqual(withoutPoolWarning(prepared.reports[0].result), withoutPoolWarning(report));
      const esmPrepared = await esm.analyzeProject(request);
      assert.deepEqual(
        withoutPoolWarning(esmPrepared.reports[0].result),
        withoutPoolWarning(report),
      );

      const current = snapshotReport(report, root);
      if (root !== nodeRoot) {
        assert.ok(current.rules.length > 0, `${label} has no live repository findings`);
        assert.ok(current.advisories.length > 0 || label === "unresolved_report");
      }
      if (root === nodeRoot) {
        const [rules, advisories] = {
          node_default: [1, 26],
          node_report: [27, 0],
          node_custom: [0, 2],
          node_advisory: [0, 27],
        }[label];
        assert.equal(current.rules.length, rules);
        assert.equal(current.advisories.length, advisories);
      } else if (label === "unresolved_report") {
        assert.ok(
          current.rules.some(({ file, line }) => file === "helpers/list-files.ts" && line === 4),
        );
      } else {
        assert.ok(
          current.advisories.some(
            ({ file, line }) => file === "helpers/list-files.ts" && line === 4,
          ),
        );
      }
      if (expected) assert.deepEqual(current, expected, `${label} differs across jobs`);
      else expected = current;
    }
    snapshots[label] = expected;
  }

  await globalThis
    .expect(`${JSON.stringify(snapshots, null, 2)}\n`)
    .toMatchFileSnapshot(snapshotPath);
}

for (const [label, config] of [
  ["default", ".no-mistakes.yml"],
  ["unresolved_report", "report.yml"],
  ["container_exports", "containers.yml"],
]) {
  const policy = label.replaceAll("_", "-");
  test(
    `compiled check and prepared analysis snapshot ${policy} live repository scans`,
    { skip: !compiled },
    async () => {
      await assertParity(
        [[label, root, config]],
        `snapshots/test-no-live-repository-scan-${policy}-api.snap`,
      );
    },
  );
}

for (const [policy, config] of [
  ["default", ".no-mistakes.yml"],
  ["report", "report.yml"],
  ["custom", "custom.yml"],
  ["advisory", "advisory.yml"],
]) {
  test(
    `compiled Node loader ${policy} scans preserve check and prepared analysis parity`,
    { skip: !compiled },
    async () => {
      await assertParity(
        [[`node_${policy}`, nodeRoot, config]],
        `snapshots/test-no-live-repository-scan-node-loaders-${policy}-api.snap`,
      );
    },
  );
}

for (const name of [
  "const-load",
  "nested-async-return",
  "owned-object-return",
  "recursive-parameter",
  "nested-recursion",
  "same-file-call-chain",
  "options-root",
  "assignment-target-options",
  "assignment-read-options",
  "glob-options-flow",
  "disposable-root",
  "rewritten-spread",
  "hook-env-restore",
  "barrel-route",
]) {
  test(
    `compiled bounded provenance ${name} preserves async check and prepared analysis parity`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const fixture = join(root, "../bounded-provenance", name);
      let expected;
      for (const jobs of [1, 2]) {
        const options = { root: fixture, config: join(fixture, ".no-mistakes.yml"), jobs };
        const request = { ...options, reports: [{ type: "check" }] };
        const calls = [
          () => cjs.check(options),
          () => esm.check(options),
          () => cjs.analyzeProject(request),
          () => esm.analyzeProject(request),
        ];
        for (const [index, call] of calls.entries()) {
          const pending = call();
          assert.equal(typeof pending.then, "function", `${name}: API ${index} is not async`);
          const result = await pending;
          const report = withoutPoolWarning(index < 2 ? result : result.reports[0].result);
          if (expected) assert.deepEqual(report, expected, `${name}: API ${index}, jobs ${jobs}`);
          else expected = report;
          const current = snapshotReport(report, fixture);
          assert.deepEqual(current.rules, [], `${name}: unexpected live repository scan`);
          const targets = current.advisories.map(({ target }) => target).sort();
          const advisories =
            {
              "assignment-target-options": Array(10).fill("unresolved root"),
              "glob-options-flow": ["unresolved root"],
              "hook-env-restore": ["unresolved root"],
              "rewritten-spread": ["runtime asset", "unresolved root"],
            }[name] ?? [];
          assert.deepEqual(targets, advisories, `${name}: unexpected provenance result`);
        }
      }
    },
  );
}
