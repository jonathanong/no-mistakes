import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname } from "./helpers.mjs";
import { check, entry, fixtureRoot, id, lineMessages } from "./forbidden-member-chains-helpers.mjs";

const dotted = (count) => Array.from({ length: count }, (_, index) => `s${index}`).join(".");
const starDotted = (count) => `*.${dotted(count)}`;

function chain(path, receiver) {
  return { path, message: "ok", ...(receiver === undefined ? {} : { receiver }) };
}

function accept(path, receiver) {
  assert.deepEqual(check("safe.mjs", { chains: [chain(path, receiver)] }), []);
}

function reject(path, receiver) {
  assert.throws(
    () => accept(path, receiver),
    /segment|maximum|malformed|required|schema|anyOf|pattern/i,
  );
}

describe(`${id} schema`, () => {
  it("rejects malformed paths instead of dropping them", () => {
    for (const path of [
      "records.*.entry",
      "records..entry",
      "records.",
      ".entry",
      "*",
      "*.",
      "*.*",
      "*.*.entry",
      dotted(17),
      starDotted(16),
    ]) {
      reject(path);
    }
    reject(["records", "*", "entry"]);
    reject(["*"]);
    reject(["*", "*"]);
    reject(Array.from({ length: 17 }, (_, index) => `s${index}`));
    reject("*", "*");
    reject(dotted(16), "root");
    reject(starDotted(16), "root");
    reject(
      Array.from({ length: 16 }, () => "a"),
      "root",
    );
    reject(["entry", "*"], "root");
    // A bad sibling must fail the whole list, not leave the valid chain active.
    assert.throws(
      () =>
        check("named.mjs", {
          chains: [
            { path: "records.*.entry", message: "middle" },
            { path: "records.entry", message: entry },
          ],
        }),
      /segment|maximum|malformed|schema|anyOf|pattern/i,
    );
  });

  it("names the empty segment and how to fix the path", () => {
    assert.throws(() => accept("records..entry"), /empty segment at index 1 in 'records\.\.entry'/);
  });

  it("accepts receiver wildcards, named roots, and the 16-segment budget", () => {
    accept("records.entry");
    accept("*.panel.open");
    accept("*.#slot");
    accept("matrix.1");
    accept("*panel.open");
    accept(dotted(16));
    accept(starDotted(15));
    accept(["drawer", "toggle"], "*");
    accept(["panel.open"], "*");
    accept(["*", "panel", "open"], "*");
    accept(dotted(15), "root");
    accept(starDotted(15), "root");
    accept("*", "root");
    accept(["*"], "root");
    accept(["*", "entry"], "records");
    accept(
      Array.from({ length: 15 }, () => "a"),
      "root",
    );
    assert.deepEqual(
      lineMessages(
        check("named.mjs", {
          chains: [{ receiver: "records", path: ["*", "entry"], message: "Named." }],
        }),
      ),
      [
        [1, "Named."],
        [3, "Named."],
      ],
    );
  });

  it("rejects the same malformed paths under oxlint", () => {
    for (const config of [
      ".oxlintrc.malformed-star.json",
      ".oxlintrc.malformed-gap.json",
      ".oxlintrc.malformed-array.json",
    ]) {
      const result = spawnSync(
        process.execPath,
        [
          resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
          // CI sets GITHUB_ACTIONS. That formatter drops the JS plugin error text,
          // so pin the default report that names the bad segment.
          "--format",
          "default",
          "--config",
          resolve(fixtureRoot, config),
          resolve(fixtureRoot, "safe.mjs"),
        ],
        { encoding: "utf8" },
      );
      assert.equal(result.error, undefined, result.stderr);
      assert.notEqual(result.status, 0, config);
      assert.match(
        `${result.stdout}\n${result.stderr}`,
        /segment|maximum|malformed|pattern|anyOf|schema|invalid/i,
        config,
      );
    }
  });
});
