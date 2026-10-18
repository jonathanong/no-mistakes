const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");
const root = join(
  __dirname,
  "..",
  "..",
  "..",
  "test-cases",
  "rules",
  "workflow-topology-policy",
  "fixture",
  "secret-inventory",
  "repository",
);

test(
  "compiled async secret inventory checks match batched analysis and ESM",
  {
    skip: !compiled,
  },
  async () => {
    const api = require("../index.js");
    const esm = await import("../index.mjs");
    const options = { root, config: ".no-mistakes.yml" };
    const pending = api.check(options);
    assert.equal(typeof pending.then, "function");
    const standalone = await pending;
    const findings = standalone.rules.filter(
      (finding) => finding.rule === "workflow-topology-policy",
    );
    assert.deepEqual(
      findings.map((finding) => finding.line),
      [8, 10, 13],
    );
    assert.ok(findings.every((finding) => finding.file === ".github/workflows/check.yml"));
    assert.deepEqual(await esm.check(options), standalone);
    const topology = await api.ciTopology({ root });
    const batchedPending = api.analyzeProject({
      ...options,
      reports: [{ type: "check" }, { type: "ciTopology" }, { type: "check" }],
    });
    assert.equal(typeof batchedPending.then, "function");
    const batched = await batchedPending;
    assert.deepEqual(
      batched.reports.map((report) => report.result),
      [standalone, topology, standalone],
    );
    const disabled = await api.check({ root, config: "disabled.yml" });
    assert.deepEqual(disabled.rules, []);
  },
);
