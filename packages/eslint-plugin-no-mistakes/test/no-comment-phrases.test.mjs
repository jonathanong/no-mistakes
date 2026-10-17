import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { fixture, lint, plugin, require, __dirname } from "./helpers.mjs";

const { ruleSuppression } = require("../src/rule-suppression");
const id = "no-comment-phrases";
const fixtureRoot = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const DEFAULT_MESSAGE =
  "This comment matches a configured phrase. Remove the preserved path, or suppress a genuine external constraint with a nonempty reason.";
// Identical comments would alternate under a stateful global regex. Zero-width
// `a*` matches every comment once, including `beta` and the empty block.
const regexLines = {
  ".oxlintrc.global.json": [1, 2, 3, 4, 5, 6],
  ".oxlintrc.sticky.json": [1, 2, 3, 5],
  ".oxlintrc.zero.json": [1, 2, 3, 4, 5, 6, 7, 8],
  ".oxlintrc.lookahead.json": [1, 2, 3, 4, 5, 6],
};
const reasonOff = [1, 12, 20, 23];
const reasonOn = [1, 5, 7, 9, 12, 20, 21, 23];

function ruleOptions(configName) {
  const config = JSON.parse(readFileSync(resolve(fixtureRoot, configName), "utf8"));
  return config.rules[`no-mistakes/${id}`][1];
}

const options = ruleOptions(".oxlintrc.json");

function check(name, ruleConfig = ["error", options]) {
  return lint(
    fixture(`${id}/${name}`),
    { [`no-mistakes/${id}`]: ruleConfig },
    resolve(fixtureRoot, name),
  );
}

function lines(name, ruleConfig) {
  return check(name, ruleConfig).map((message) => message.line);
}

function loc(line) {
  return { start: { line, column: 0 }, end: { line, column: 1 } };
}

function oxlint(configName, name) {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(fixtureRoot, configName),
      "--format",
      "json",
      resolve(fixtureRoot, name),
    ],
    { encoding: "utf8" },
  );
  const report = JSON.parse(result.stdout || "{}");
  const diagnostics = (report.diagnostics || []).filter((entry) => entry.code?.includes(id));
  assert.equal(
    result.status,
    diagnostics.length ? 1 : 0,
    `${name}\n${result.stderr}\n${result.stdout}`,
  );
  return diagnostics;
}

function oxlintLines(configName, name) {
  return oxlint(configName, name).map((entry) => entry.labels[0].span.line);
}

describe(id, () => {
  it("flags configured line, block, and JSX comments and ignores strings", () => {
    assert.deepEqual(lines("comments.mjs"), [1, 3, 4, 5]);
    assert.deepEqual(lines("jsx.jsx"), [1]);
    assert.deepEqual(lines("strings.ts"), []);
    assert.deepEqual(lines("shebang.mjs"), [2]);
    assert.deepEqual(lines("comments.mjs", "error"), []);
    assert.deepEqual(lines("comments.mjs", ["error", {}]), []);
    assert.deepEqual(lines("comments.mjs", ["error", { patterns: [] }]), []);
  });

  it("uses a scoped pattern message, then the top-level message, then the default", () => {
    const findings = check("messages.mjs");
    assert.deepEqual(
      findings.map((message) => message.message),
      [options.patterns[0].message, options.message],
    );
    assert.equal(
      check("messages.mjs", ["error", { patterns: [{ pattern: "configured-fallback-phrase" }] }])[0]
        .message,
      DEFAULT_MESSAGE,
    );
    assert.equal(
      check("messages.mjs", [
        "error",
        { patterns: [{ pattern: "configured-fallback-phrase" }], message: "Top level only" },
      ])[0].message,
      "Top level only",
    );
    assert.equal(
      check("comments.mjs", [
        "error",
        {
          patterns: [{ pattern: "kept for compatibility", message: "Scoped kept" }],
          message: "Top level only",
        },
      ])[0].message,
      "Scoped kept",
    );
  });

  it("rejects invalid patterns and still applies valid siblings", () => {
    const invalidOnly = check("comments.mjs", [
      "error",
      { patterns: [{ pattern: "[", flags: "i" }] },
    ]);
    assert.equal(invalidOnly.length, 1);
    assert.equal(invalidOnly[0].messageId, "invalidPattern");
    assert.match(invalidOnly[0].message, /\[/);
    const both = check("invalid.mjs", ["error", ruleOptions(".oxlintrc.invalid.json")]);
    assert.deepEqual(
      both.map((message) => message.messageId),
      ["invalidPattern", "phrase"],
    );
    assert.equal(both[1].line, 1);
  });

  it("defines global, sticky, and zero-width matches once per comment", () => {
    for (const [configName, expected] of Object.entries(regexLines)) {
      assert.deepEqual(
        lines("regex.mjs", ["error", ruleOptions(configName)]),
        expected,
        configName,
      );
    }
    assert.deepEqual(lines("case.mjs"), [2]);
    assert.deepEqual(lines("case.mjs", ["error", ruleOptions(".oxlintrc.case.json")]), [1, 2]);
  });

  it("requires a nonempty reason only when requireSuppressionReason is on", () => {
    assert.deepEqual(lines("reasons.mjs"), reasonOff);
    assert.deepEqual(
      lines("reasons.mjs", ["error", { ...options, requireSuppressionReason: false }]),
      reasonOff,
    );
    assert.deepEqual(
      lines("reasons.mjs", ["error", { ...options, requireSuppressionReason: true }]),
      reasonOn,
    );
    assert.deepEqual(lines("suppression-file.mjs"), []);
    assert.deepEqual(
      lines("suppression-file.mjs", ["error", { ...options, requireSuppressionReason: true }]),
      [],
    );
    assert.deepEqual(lines("suppression-file-bare.mjs"), []);
    assert.deepEqual(
      lines("suppression-file-bare.mjs", ["error", { ...options, requireSuppressionReason: true }]),
      [2, 3],
    );
    assert.deepEqual(lines("suppression-file-blank.mjs"), []);
    assert.deepEqual(
      lines("suppression-file-blank.mjs", [
        "error",
        { ...options, requireSuppressionReason: true },
      ]),
      [4],
    );
    assert.deepEqual(lines("suppression-file-block.mjs"), []);
    assert.deepEqual(
      lines("suppression-file-block.mjs", [
        "error",
        { ...options, requireSuppressionReason: true },
      ]),
      [],
    );
    assert.deepEqual(lines("late-file.mjs"), [3, 4]);
    assert.deepEqual(
      lines("late-file.mjs", ["error", { ...options, requireSuppressionReason: true }]),
      [3, 4],
    );
  });

  it("keeps the shared suppression helper unchanged unless a caller opts in", () => {
    const context = {
      sourceCode: {
        ast: { tokens: [{ range: [100, 101] }] },
        getAllComments: () => [
          {
            type: "Line",
            value: " no-mistakes-disable-next-line demo",
            loc: loc(1),
            range: [0, 10],
          },
          {
            type: "Line",
            value: " no-mistakes-disable-next-line demo --   ",
            loc: loc(3),
            range: [0, 10],
          },
          {
            type: "Line",
            value: " no-mistakes-disable-next-line demo -- because",
            loc: loc(5),
            range: [0, 10],
          },
        ],
      },
    };
    const without = ruleSuppression(context, "demo");
    const withReason = ruleSuppression(context, "demo", { requireReason: true });
    assert.deepEqual(
      [2, 4, 6].map((line) => without({ loc: loc(line) })),
      [true, true, true],
    );
    assert.deepEqual(
      [2, 4, 6].map((line) => withReason({ loc: loc(line) })),
      [false, false, true],
    );
  });

  it("compiles each pattern once and skips shebang comment tokens", () => {
    const RealRegExp = globalThis.RegExp;
    let constructs = 0;
    globalThis.RegExp = class extends RealRegExp {
      constructor(...args) {
        constructs += 1;
        super(...args);
      }
    };
    try {
      const reports = [];
      const context = {
        options: [{ patterns: [{ pattern: "alpha", flags: "g" }] }],
        sourceCode: {
          ast: { tokens: [{ range: [0, 1] }] },
          getAllComments: () => [
            { type: "Line", value: "alpha", loc: loc(1), range: [0, 1] },
            { type: "Line", value: "alpha", loc: loc(2), range: [0, 1] },
            { type: "Shebang", value: "alpha", loc: loc(3), range: [0, 1] },
            { type: "Block", value: "alpha", loc: loc(4), range: [0, 1] },
          ],
        },
        report(descriptor) {
          reports.push(descriptor);
        },
      };
      plugin.rules[id].create(context).Program({ loc: loc(1) });
      assert.equal(constructs, 1);
      assert.deepEqual(
        reports.map((report) => report.loc.start.line),
        [1, 2, 4],
      );
    } finally {
      globalThis.RegExp = RealRegExp;
    }
  });

  it("rejects schema violations for this rule's options", () => {
    for (const ruleConfig of [
      ["error", { requireSuppressionReason: "yes" }],
      ["error", { extra: true }],
      ["error", { patterns: [{ pattern: "" }] }],
      ["error", { patterns: [{}] }],
      ["error", { patterns: [{ pattern: "alpha", flags: 1 }] }],
      ["error", { patterns: [{ pattern: "x", flags: "gg" }] }],
      ["error", { patterns: [{ pattern: "x", flags: "z" }] }],
      ["error", { patterns: [{ pattern: "x", flags: "gimgi" }] }],
      ["error", { patterns: [{ pattern: "x", flags: "G" }] }],
    ]) {
      assert.throws(() => check("strings.ts", ruleConfig), /Key "rules"/);
    }
    assert.throws(
      () => check("strings.ts", ["error", { patterns: [{ pattern: "x", flags: "gg" }] }]),
      /should match pattern/,
    );
    assert.deepEqual(
      lines("strings.ts", ["error", { patterns: [{ pattern: "alpha", flags: "" }] }]),
      [],
    );
    assert.deepEqual(
      lines("strings.ts", ["error", { patterns: [{ pattern: "alpha", flags: "im" }] }]),
      [],
    );
    const schema = plugin.rules[id].meta.schema[0];
    const flagPattern = new RegExp(schema.properties.patterns.items.properties.flags.pattern);
    assert.equal(schema.additionalProperties, false);
    assert.equal(schema.properties.patterns.maxItems, 64);
    assert.deepEqual(schema.properties.patterns.items.required, ["pattern"]);
    assert.equal(schema.properties.requireSuppressionReason.type, "boolean");
    assert.equal(flagPattern.source, "^(?!.*(.).*\\1)[dgimsuvy]*$");
    for (const flags of ["", "d", "g", "i", "m", "s", "u", "v", "y", "gi", "yu", "dgimsuvy"]) {
      assert.equal(flagPattern.test(flags), true, flags);
    }
    for (const flags of ["gg", "z", "G", "gimgi", "g ", "ii", "x"]) {
      assert.equal(flagPattern.test(flags), false, flags);
    }
  });

  it("rejects invalid flags when the oxlint config loads", () => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(fixtureRoot, ".oxlintrc.bad-flags.json"),
        "--format",
        "json",
        resolve(fixtureRoot, "comments.mjs"),
        resolve(fixtureRoot, "strings.ts"),
      ],
      { encoding: "utf8" },
    );
    const output = `${result.stderr}\n${result.stdout}`;
    assert.notEqual(result.status, 0, output);
    assert.match(output, /no-mistakes\/no-comment-phrases/);
    assert.match(output, /gg/);
    assert.match(output, /should match pattern/);
    assert.doesNotMatch(output, /Invalid comment phrase pattern/);
  });

  it("has real oxlint parity for comments, strings, regex, and reasons", () => {
    const expected = [
      [".oxlintrc.json", "comments.mjs", [1, 3, 4, 5]],
      [".oxlintrc.json", "strings.ts", []],
      [".oxlintrc.json", "jsx.jsx", [1]],
      [".oxlintrc.json", "shebang.mjs", [2]],
      [".oxlintrc.json", "messages.mjs", [1, 2]],
      [".oxlintrc.json", "regex.mjs", []],
      [".oxlintrc.json", "reasons.mjs", reasonOff],
      [".oxlintrc.json", "suppression-file.mjs", []],
      [".oxlintrc.json", "suppression-file-bare.mjs", []],
      [".oxlintrc.json", "suppression-file-blank.mjs", []],
      [".oxlintrc.json", "suppression-file-block.mjs", []],
      [".oxlintrc.json", "late-file.mjs", [3, 4]],
      [".oxlintrc.json", "case.mjs", [2]],
      [".oxlintrc.reason.json", "reasons.mjs", reasonOn],
      [".oxlintrc.reason.json", "suppression-file.mjs", []],
      [".oxlintrc.reason.json", "suppression-file-bare.mjs", [2, 3]],
      [".oxlintrc.reason.json", "suppression-file-blank.mjs", [4]],
      [".oxlintrc.reason.json", "suppression-file-block.mjs", []],
      [".oxlintrc.reason.json", "late-file.mjs", [3, 4]],
      ...Object.entries(regexLines).map(([configName, expectedLines]) => [
        configName,
        "regex.mjs",
        expectedLines,
      ]),
      [".oxlintrc.case.json", "case.mjs", [1, 2]],
      [".oxlintrc.invalid.json", "invalid.mjs", [1, 1]],
    ];
    for (const [configName, name, expectedLines] of expected) {
      assert.deepEqual(oxlintLines(configName, name), expectedLines, `${configName} ${name}`);
    }
    const messages = oxlint(".oxlintrc.json", "messages.mjs").map((entry) => entry.message);
    assert.deepEqual(messages, [options.patterns[0].message, options.message]);
    assert.ok(
      oxlint(".oxlintrc.invalid.json", "invalid.mjs").some((entry) =>
        /Invalid comment phrase pattern/.test(entry.message),
      ),
    );
  }, 30_000);

  it("exports an opt-in rule without an unsafe fix", () => {
    assert.ok(plugin.rules[id]);
    assert.equal(plugin.rules[id].meta.docs.recommended, false);
    assert.equal(plugin.rules[id].meta.fixable, undefined);
    assert.equal(plugin.configs.recommended.rules[`no-mistakes/${id}`], undefined);
    assert.equal(plugin.configs.strict.rules[`no-mistakes/${id}`], undefined);
  });
});
