const assert = require("node:assert/strict");
const { join } = require("node:path");
const { readFileSync } = require("node:fs");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled checks honor exact effect bindings through aliases and helper traces",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/targets-fixture",
    );
    const report = await cjs.check({ root });
    assert.deepEqual(report.warnings, []);
    const findings = report.rules.filter((finding) => finding.rule === "query-reached-per-item");
    assert.deepEqual(
      findings.map((finding) => finding.line).sort((a, b) => a - b),
      [11, 12, 13, 14, 16, 18, 19, 20, 21, 23, 24, 25, 26, 27, 28, 29],
      "only configured module bindings should report; streams, shadows and local parameters stay quiet",
    );
    assert.ok(findings.every((finding) => finding.file === "src/entry.mts"));
    assert.ok(findings.some((finding) => finding.message.includes("lookup → inner → load")));
    assert.ok(
      findings.some(
        (finding) =>
          finding.message.includes("cachedLookup → valkey.hget") &&
          finding.message.includes("reaches `valkey`"),
      ),
    );
    assert.deepEqual(await esm.check({ root }), report);
    const batch = await cjs.analyzeProject({ root, reports: [{ type: "check" }] });
    assert.deepEqual(batch.reports[0].result, report);
  },
);

test(
  "compiled CJS, ESM and batched checks share per-item effect diagnostics",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(__dirname, "../../../test-cases/rules/query-reached-per-item/fixture");
    const report = await cjs.check({ root });
    assert.deepEqual(report.warnings, []);
    const findings = report.rules.filter((finding) => finding.rule === "query-reached-per-item");
    assert.ok(
      findings.some(
        (finding) => finding.line === 5 && finding.message.includes("lookup → inner → read"),
      ),
    );
    assert.ok(
      findings.some(
        (finding) => finding.line === 21 && finding.message.includes("transaction client only"),
      ),
    );
    assert.ok(
      !findings.some(
        (finding) => finding.file === "src/entry.mts" && [23, 26, 28, 29].includes(finding.line),
      ),
    );
    assert.ok(
      findings.some((finding) => finding.file === "src/entry.mts" && finding.line === 24),
      "eager queries execute before a batch wrapper receives their results",
    );
    const aliasLines = readFileSync(join(root, "src/entry.mts"), "utf8")
      .split("\n")
      .flatMap((line, index) =>
        /items\.(?:map|forEach)\(lookupAlias\)/.test(line) ? [index + 1] : [],
      );
    assert.equal(aliasLines.length, 2);
    for (const line of aliasLines) {
      assert.ok(
        findings.some((finding) => finding.file === "src/entry.mts" && finding.line === line),
        `named callback alias at line ${line} must reach its configured sink`,
      );
    }
    assert.deepEqual(await esm.check({ root }), report);
    const batch = await cjs.analyzeProject({ root, reports: [{ type: "check", id: "per-item" }] });
    assert.equal(batch.reports[0].id, "per-item");
    assert.deepEqual(batch.reports[0].result, report);
    const callOptions = { file: "src/helpers.mts", exportName: "lookup" };
    const callSites = await cjs.callSites({ root, ...callOptions });
    const combined = await cjs.analyzeProject({
      root,
      reports: [
        { type: "check", id: "per-item" },
        { type: "callSites", id: "ordinary-calls", ...callOptions },
      ],
    });
    assert.deepEqual(
      combined.reports.find((result) => result.id === "ordinary-calls").result,
      callSites,
      "adding the per-item check must preserve ordinary call-site fields",
    );
    const suppressed = await cjs.check({ root, includeSuppressed: true });
    assert.deepEqual(suppressed.rules, report.rules);
    assert.ok(suppressed.suppressed.length >= 2);
  },
);
