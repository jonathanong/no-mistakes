const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
test(
  "compiled async guarded array effects retain both receiver candidates",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-receiver-escapes/fixture",
    );
    const options = {
      root,
      config: join(root, "effects.yml"),
      kind: "receiver",
      entry: "src/array-reassign-conditional.mts",
    };
    const pending = cjs.effects(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    // Keep all four old matches and recover the guarded replacement's original method.
    assert.deepEqual(
      report.callSites.map(({ line, callee }) => [line, callee]),
      [
        [12, "replica.get"],
        [13, "untouched.get"],
        [14, "primary.get"],
        [15, "replica.info"],
        [16, "unrelated.get"],
      ],
    );
    assert.deepEqual(await esm.effects(options), report);
    for (const api of [cjs, esm]) {
      const batch = api.analyzeProject({
        root,
        config: options.config,
        reports: [{ type: "effects", kind: "receiver", entry: options.entry }],
      });
      assert.ok(batch instanceof Promise);
      assert.deepEqual((await batch).reports[0].result, report);
    }
  },
);

test(
  "compiled async guarded parameter captures retain the original unaffected method",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-guarded-parameters/fixture",
    );
    const scenarios = [
      [
        "before",
        [
          [12, "replica.get"],
          [14, "primary.get"],
          [15, "replica.info"],
          [16, "unrelated.get"],
        ],
      ],
      [
        "after",
        [
          [13, "untouched.get"],
          [14, "primary.get"],
          [15, "replica.info"],
          [16, "unrelated.get"],
        ],
      ],
      [
        "destructure",
        [
          [12, "replica.get"],
          [13, "untouched.get"],
          [14, "primary.get"],
          [15, "replica.info"],
          [16, "unrelated.get"],
        ],
      ],
    ];
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: scenarios.map(([suffix]) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/array-latest-guarded-parameter-${suffix}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const [index, [, expected]] of scenarios.entries()) {
      assert.deepEqual(
        batch.reports[index].result.callSites.map(({ line, callee }) => [line, callee]),
        expected,
      );
    }
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);

test(
  "compiled async recovered patterns and optional widths preserve prior candidates",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [suite, entries] of [
      ["value-flow-guarded-parameters", ["array-parameter-destructure-unguarded"]],
      [
        "value-flow-optional-prefixes",
        ["array-optional-push-after-empty-prefix", "array-optional-push-after-wide-prefix"],
      ],
      [
        "value-flow-receiver-escapes",
        [
          "array-optional-push-then-push",
          "array-optional-push-then-suffix",
          "array-optional-push-forwarded",
          "array-optional-push-function-call",
          "array-optional-push-method-call",
        ],
      ],
    ]) {
      const root = join(__dirname, "../../../test-cases/codebase-analysis", suite, "fixture");
      const options = {
        root,
        config: join(root, "effects.yml"),
        reports: entries.map((entry) => ({
          type: "effects",
          kind: "receiver",
          entry: `src/${entry}.mts`,
        })),
      };
      const pending = cjs.analyzeProject(options);
      assert.ok(pending instanceof Promise);
      const batch = await pending;
      for (const { result } of batch.reports) {
        assert.deepEqual(
          result.callSites.map(({ callee }) => callee),
          ["replica.get", "untouched.get", "primary.get", "replica.info", "unrelated.get"],
        );
      }
      assert.deepEqual(await esm.analyzeProject(options), batch);
    }
  },
);

test(
  "compiled async fixed prefixes preserve intervening write origins",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-optional-prefixes/fixture",
    );
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: [
        "array-optional-push-after-fixed-prefix",
        "array-optional-push-after-fixed-prefix-call",
        "array-optional-push-after-fixed-prefix-forwarded",
      ].map((entry) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/${entry}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const { result } of batch.reports) {
      // Both flag states replace replica.get locally: retaining a vendor edge is an over-claim.
      assert.deepEqual(
        result.callSites.map(({ callee }) => callee),
        ["untouched.get", "primary.get", "replica.info", "unrelated.get"],
      );
    }
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);
