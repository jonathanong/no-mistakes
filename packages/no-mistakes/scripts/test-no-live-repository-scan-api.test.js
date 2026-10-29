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
          node_default: [1, 23],
          node_report: [24, 0],
          node_custom: [0, 2],
          node_advisory: [0, 24],
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

test(
  "compiled check and prepared analysis snapshot live repository scans",
  { skip: !compiled },
  async () => {
    await assertParity(
      [
        ["default", root, ".no-mistakes.yml"],
        ["unresolved_report", root, "report.yml"],
        ["container_exports", root, "containers.yml"],
      ],
      "snapshots/test-no-live-repository-scan-api.snap",
    );
  },
);

test(
  "compiled Node loader scans preserve check and prepared analysis parity",
  { skip: !compiled },
  async () => {
    await assertParity(
      [
        ["node_default", nodeRoot, ".no-mistakes.yml"],
        ["node_report", nodeRoot, "report.yml"],
        ["node_custom", nodeRoot, "custom.yml"],
        ["node_advisory", nodeRoot, "advisory.yml"],
      ],
      "snapshots/test-no-live-repository-scan-node-loaders-api.snap",
    );
  },
);
