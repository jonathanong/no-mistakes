const assert = require("node:assert/strict");
const { dirname, join } = require("node:path");
const { spawnSync } = require("node:child_process");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled async default callbacks preserve capture generations",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-guarded-parameters/fixture",
    );
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: ["before", "after"].map((suffix) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/array-parameter-default-callback-${suffix}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const [index, retained] of ["replica.get", "untouched.get"].entries()) {
      // The default callable must receive its input before the capture is frozen.
      assert.deepEqual(
        batch.reports[index].result.callSites.map(({ callee }) => callee),
        [retained, "primary.get", "replica.info", "unrelated.get"],
      );
    }
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);

test(
  "compiled async bound array spreads retain candidates and exact empty controls",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-optional-prefixes/fixture",
    );
    const scenarios = [
      ["array-bind-varying-width", null],
      ["array-bind-optional-push", null],
      ["array-bind-known-empty", "untouched.get"],
      ["array-bind-packed-first-spread", "replica.get"],
      ["array-bind-empty-first-spread", "untouched.get"],
      ["array-bind-multi-value-spread-suffix", "replica.get"],
      ["array-bind-conditional-first-spread", null],
    ];
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: scenarios.map(([entry]) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/${entry}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const [index, { result }] of batch.reports.entries()) {
      assert.deepEqual(
        result.callSites.map(({ callee }) => callee),
        ["replica.get", "untouched.get", "primary.get", "replica.info", "unrelated.get"].filter(
          (callee) => callee !== scenarios[index][1],
        ),
      );
    }
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);

test(
  "compiled async assigned callbacks retain late origins in nonempty captures",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-guarded-parameters/fixture",
    );
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: ["before", "after"].map((suffix) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/array-forwarded-late-captured-${suffix}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const { result } of batch.reports)
      assert.deepEqual(
        result.callSites.map(({ callee }) => callee),
        ["primary.get", "replica.info", "unrelated.get"],
      );
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);

test(
  "compiled async conditional callbacks keep late captured receiver candidates",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-guarded-parameters/fixture",
    );
    for (const [suffix, firstLine] of [
      ["conditional", 20],
      ["conditional-chain", 21],
      ["member-conditional", 21],
      ["inner-conditional", 20],
      ["inner-loop", 20],
      ["inner-return", 21],
      ["forward-return", 21],
      ["assignment-conditional", 20],
      ["direct-conditional", 16],
    ]) {
      const entry = `src/array-forwarded-late-captured-${suffix}.mts`;
      const options = { root, config: join(root, "effects.yml"), kind: "receiver", entry };
      const pending = cjs.effects(options);
      assert.ok(pending instanceof Promise);
      const expected = {
        kind: "receiver",
        entry,
        callSites: [
          { file: entry, line: firstLine, callee: "untouched.get", depth: 0 },
          { file: entry, line: firstLine + 1, callee: "primary.get", depth: 0 },
          { file: entry, line: firstLine + 2, callee: "replica.info", depth: 0 },
          { file: entry, line: firstLine + 3, callee: "unrelated.get", depth: 0 },
        ],
        byCategory: { uncategorized: 4 },
      };
      assert.deepEqual(await pending, expected);
      assert.deepEqual(await esm.effects(options), expected);
      const batch = await cjs.analyzeProject({
        root,
        config: options.config,
        reports: [{ type: "effects", kind: "receiver", entry }],
      });
      assert.deepEqual(batch.reports[0].result, expected);
      const cli =
        process.env.NO_MISTAKES_TEST_CLI_PATH ||
        join(
          dirname(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH),
          process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes",
        );
      const child = spawnSync(
        cli,
        [
          "effects",
          "receiver",
          "--entry",
          entry,
          "--root",
          root,
          "--config",
          options.config,
          "--format",
          "json",
        ],
        { encoding: "utf8", env: process.env },
      );
      assert.equal(child.error, undefined);
      assert.equal(child.status, 0, child.stderr);
      assert.deepEqual(JSON.parse(child.stdout), expected);
    }
  },
);
