const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { isAbsolute, join, relative, resolve, sep } = require("node:path");
const test = globalThis.test || require("node:test").test;
const { fixtureRelativePath, normalizeFixtureReport } = require("./fixture-relative-path.js");

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

for (const [name, advisories] of [
  ["same-test-owned", 0],
  ["same-test-repository", 1],
  ["same-test-conditional", 1],
  ["same-test-restored", 0],
  ["same-test-unrelated-write", 0],
  ["same-test-reset-between", 1],
  ["same-test-helper-reset", 1],
  ["same-test-default-reset", 1],
  ["same-test-argument-reset", 1],
  ["same-test-await-between", 1],
  ["same-test-unknown-vi", 1],

  ["stub-direct", 0],
  ["assignment-direct", 0],
  ["stub-setup", 0],
  ["assignment-setup", 0],
  ["configured-stub", 0],
  ["configured-assignment", 0],
  ["transitive-setup", 0],
  ["unstubbed", 1],
  ["repository", 1],
  ["mocked-setup", 1],
  ["typed-mocked-setup", 1],
]) {
  test(
    `compiled owned env ${name} preserves async check and prepared analysis parity`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const fixture = join(root, "../env-provenance", name);
      let expected;
      for (const jobs of [1, 2]) {
        const options = { root: fixture, config: join(fixture, ".no-mistakes.yml"), jobs };
        const request = { ...options, reports: [{ type: "check" }] };
        for (const [index, call] of [
          () => cjs.check(options),
          () => esm.check(options),
          () => cjs.analyzeProject(request),
          () => esm.analyzeProject(request),
        ].entries()) {
          const pending = call();
          assert.equal(typeof pending.then, "function", `${name}: API ${index} is not async`);
          const result = await pending;
          const report = withoutPoolWarning(index < 2 ? result : result.reports[0].result);
          if (expected) assert.deepEqual(report, expected, `${name}: API ${index}, jobs ${jobs}`);
          else expected = report;
          assert.deepEqual(report.rules, [], `${name}: unexpected live repository scan`);
          assert.equal(report.advisories.length, advisories, name);
        }
      }
    },
  );
}

for (const [name, expected, incomplete = []] of [
  ["minimal", []],
  ["production-only", []],
  ["large-unknown", []],
  ["large-collision", []],
  ["small", ["src/scan.ts:21", "src/scan.ts:25", "src/scan.ts:29", "src/scan.ts:33"]],
  [
    "large",
    [
      "src/long-chain.ts:260",
      "src/scan.ts:21",
      "src/scan.ts:25",
      "src/scan.ts:29",
      "src/scan.ts:33",
    ],
  ],
  ["no-caller", []],
  ["disabled", []],
  ["unknown-setup", ["src/pb.mts:5"]],
  ["unparsed-setup", [], ["setup.ts"]],
  ["unparsed-config-import", [], ["settings.ts"]],
  ["env-argument-fixture", ["src/scan.mts:13", "src/scan.mts:17"]],
  ["env-argument-poison", ["src/scan.mts:5"]],
  ["env-argument-captured", ["src/scan.mts:5", "src/scan.mts:9"]],
  ["env-segment-contract", ["src/scan.mts:9"]],
  ["setup-caller", ["src/list.ts:4"]],
  ["setup-hook-caller", ["src/list.ts:4"]],
  ["setup-only-live", ["src/list.ts:4"]],
  ["setup-only-owned", []],
  ["custom-config-unstub", ["src/list.ts:4"]],
  ["imported-config-unstub", ["src/list.ts:4"]],
  ["projection-config-diagnostics", ["src/list.ts:4"]],
  ["unrelated-conventional-config", []],
]) {
  test(
    `compiled caller-owned roots ${name} preserve async and prepared parity`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const fixture = join(root, "../caller-roots", name);
      let first;
      for (const jobs of [1, 2]) {
        const options = { root: fixture, config: join(fixture, ".no-mistakes.yml"), jobs };
        const request = { ...options, reports: [{ type: "check" }] };
        for (const [index, call] of [
          () => cjs.check(options),
          () => esm.check(options),
          () => cjs.analyzeProject(request),
          () => esm.analyzeProject(request),
        ].entries()) {
          const pending = call();
          assert.equal(typeof pending.then, "function");
          const result = await pending;
          const report = withoutPoolWarning(index < 2 ? result : result.reports[0].result);
          const parityReport = normalizeFixtureReport(report, fixture);
          if (first) assert.deepEqual(parityReport, first, `${name}: API ${index}, jobs ${jobs}`);
          else first = parityReport;
          const findings = [...report.rules, ...report.advisories]
            .filter((finding) => finding.rule === rule)
            .map(({ file, line }) => `${file}:${line}`)
            .sort();
          assert.deepEqual(findings, expected, name);
          assert.deepEqual(
            (report.incomplete || []).map(({ file }) => fixtureRelativePath(file, fixture)).sort(),
            incomplete,
            name,
          );
        }
      }
    },
  );
}

for (const [label, reports] of [
  [
    "ordinary graph",
    [
      { type: "importers", id: "importers", file: "settings.ts" },
      {
        type: "dependencies",
        id: "forward",
        files: ["src/config-client.ts"],
        relationships: ["import"],
        depth: 3,
      },
      {
        type: "dependents",
        id: "reverse",
        files: ["settings.ts"],
        relationships: ["import"],
        depth: 3,
      },
      {
        type: "related",
        id: "related",
        files: ["settings.ts"],
        relationships: ["import"],
        depth: 3,
      },
      {
        type: "dependents",
        id: "symbol",
        files: [{ file: "settings.ts", symbol: "resetPolicy" }],
        relationships: ["import"],
        depth: 3,
      },
    ],
  ],
  [
    "explicit config root",
    [
      {
        type: "dependents",
        id: "explicit",
        files: ["qa.runner.ts"],
        relationships: ["import"],
        depth: 3,
      },
    ],
  ],
  [
    "shared explicit root",
    [
      {
        type: "dependents",
        id: "reverse",
        files: ["settings.ts"],
        relationships: ["import"],
        depth: 3,
      },
      {
        type: "dependents",
        id: "explicit",
        files: ["qa.runner.ts"],
        relationships: ["import"],
        depth: 3,
      },
    ],
  ],
]) {
  test(
    `compiled caller-owned check preserves ${label} projection`,
    { skip: !compiled },
    async () => {
      const fixture = join(root, "../caller-roots/imported-config-unstub");
      for (const api of [require("../index.js"), await import("../index.mjs")]) {
        for (const jobs of [1, 2]) {
          const options = { root: fixture, config: join(fixture, ".no-mistakes.yml"), jobs };
          const before = await api.analyzeProject({ ...options, reports });
          const after = await api.analyzeProject({
            ...options,
            reports: [...reports, { type: "check", id: "check" }],
          });
          const stable = (items) =>
            items.map(({ result, ...item }) => ({ ...item, result: withoutPoolWarning(result) }));
          assert.deepEqual(
            stable(after.reports.slice(0, reports.length)),
            stable(before.reports),
            label,
          );
          const expected =
            label === "ordinary graph"
              ? []
              : label === "explicit config root"
                ? ["src/config-client.ts"]
                : ["qa.runner.ts", "src/config-client.ts"];
          const reverse = before.reports.find(({ id }) => id === "reverse" || id === "explicit");
          assert.deepEqual(
            reverse.result.files.map(({ path }) => fixtureRelativePath(path, fixture)),
            expected,
          );
          if (label === "ordinary graph") {
            assert.deepEqual(before.reports[0].result.directImporters, ["qa.runner.ts"]);
            assert.deepEqual(
              before.reports[1].result.files.map(({ path }) => fixtureRelativePath(path, fixture)),
              ["qa.runner.ts", "settings.ts"],
            );
            assert.deepEqual(before.reports[3].result.files, []);
          }
          const check = after.reports.at(-1).result;
          assert.deepEqual(
            check.advisories.map(({ file, line }) => `${file}:${line}`),
            ["src/list.ts:4"],
          );
        }
      }
    },
  );
}

for (const [label, type, file, diagnostics] of [
  ["reverse", "dependents", "settings.ts", []],
  ["unrelated forward", "dependencies", "src/list.ts", []],
  ["reachable forward", "dependencies", "src/config-client.ts", []],
  ["explicit root", "dependents", "qa.runner.ts", ["qa.runner.ts"]],
]) {
  test(
    `compiled caller-owned check preserves ${label} diagnostics`,
    { skip: !compiled },
    async () => {
      const fixture = join(root, "../caller-roots/projection-config-diagnostics");
      const report = { type, files: [file], relationships: ["import"], depth: 3 };
      for (const api of [require("../index.js"), await import("../index.mjs")]) {
        for (const jobs of [1, 2]) {
          const options = { root: fixture, config: join(fixture, ".no-mistakes.yml"), jobs };
          const before = await api.analyzeProject({ ...options, reports: [report] });
          const after = await api.analyzeProject({
            ...options,
            reports: [report, { type: "check" }],
          });
          assert.deepEqual(
            withoutPoolWarning(after.reports[0].result),
            withoutPoolWarning(before.reports[0].result),
            label,
          );
          assert.deepEqual(
            before.reports[0].result.diagnostics.map(({ file }) =>
              fixtureRelativePath(file, fixture),
            ),
            diagnostics,
          );
          assert.deepEqual(
            after.reports[1].result.advisories.map(({ file, line }) => `${file}:${line}`),
            ["src/list.ts:4"],
          );
        }
      }
    },
  );
}
