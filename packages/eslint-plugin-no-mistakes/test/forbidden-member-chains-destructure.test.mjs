import assert from "node:assert/strict";
import { describe, it } from "vitest";
import {
  check,
  drawer,
  entry,
  lineMessages,
  oxlintFiles,
  panel,
  slot,
} from "./forbidden-member-chains-helpers.mjs";

describe("forbidden-member-chains destructure", () => {
  it("matches object patterns, renames, nesting, and defaults", () => {
    assert.deepEqual(lineMessages(check("destructure.mjs")), [
      [2, panel],
      [6, panel],
      [9, panel],
      [12, panel],
      [15, panel],
      [18, panel],
      [21, panel],
      [24, entry],
      [27, entry],
      [31, entry],
      [48, panel],
      [54, drawer],
      [57, panel],
      [60, panel],
      [63, slot],
      [64, slot],
      [67, panel],
      [70, panel],
      [72, panel],
      [76, panel],
      [85, panel],
      [92, panel],
      [95, panel],
      [98, panel],
      [102, panel],
      [110, panel],
    ]);
    assert.deepEqual(lineMessages(check("destructure.ts")), [
      [2, panel],
      [5, entry],
      [8, panel],
    ]);
  });

  it("matches a nested default at any depth", () => {
    assert.deepEqual(lineMessages(check("destructure-nested-default.mjs")), [
      [3, panel],
      [6, panel],
      [8, panel],
    ]);
    assert.deepEqual(
      oxlintFiles(["destructure-nested-default.mjs"])
        .get("destructure-nested-default.mjs")
        .map((diagnostic) => diagnostic.message),
      check("destructure-nested-default.mjs").map((message) => message.message),
    );
  });

  it("reports one shared property and an array-pattern default", () => {
    const names = ["destructure-shared-report.mjs", "destructure-array-default.mjs"];
    const expected = {
      "destructure-shared-report.mjs": [
        [2, panel],
        [4, drawer],
      ],
      "destructure-array-default.mjs": [
        [1, panel],
        [2, panel],
      ],
    };
    const diagnostics = oxlintFiles(names);
    for (const name of names) {
      const messages = check(name);
      assert.deepEqual(lineMessages(messages), expected[name], name);
      assert.deepEqual(
        diagnostics.get(name).map((diagnostic) => diagnostic.message),
        messages.map((message) => message.message),
        name,
      );
    }
  });

  it("matches a call receiver and a shadowed named root", () => {
    assert.deepEqual(
      lineMessages(
        check("destructure-wild.mjs", {
          chains: [
            { path: "records.entry", message: "Exact." },
            { path: "*.entry", message: "Wild." },
          ],
        }),
      ),
      [
        [2, "Wild."],
        [5, "Exact."],
        [5, "Wild."],
        [8, "Wild."],
        [12, "Exact."],
        [12, "Wild."],
        [13, "Exact."],
        [13, "Wild."],
      ],
    );
  });

  it("matches oxlint on destructure fixtures", () => {
    const names = ["destructure.mjs", "destructure.ts"];
    const diagnostics = oxlintFiles(names);
    for (const name of names) {
      assert.deepEqual(
        diagnostics.get(name).map((diagnostic) => diagnostic.message),
        check(name).map((message) => message.message),
        name,
      );
    }
  });
});
