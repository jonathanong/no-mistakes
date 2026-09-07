import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

const RULE = "no-inline-noop-promise-catch";

function ruleFixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", RULE, "fixture", name),
    "utf8",
  );
}

const NOOP = "noopCatch";
const INVALID_COUNT = 19;
const OPTIONS_COUNT = 4;

describe(RULE, () => {
  it("allows named handlers and callbacks with observable behavior", () => {
    assert.deepEqual(messages(ruleFixture("valid.ts"), RULE, undefined, "valid.ts"), []);
  });

  it("reports inline no-op Promise catch callbacks", () => {
    assert.deepEqual(
      messages(ruleFixture("invalid.ts"), RULE, undefined, "invalid.ts"),
      Array(INVALID_COUNT).fill(NOOP),
    );
  });

  it("checks every file when path patterns are omitted", () => {
    assert.deepEqual(
      messages(ruleFixture("options.ts"), RULE, undefined, "web/app.ts"),
      Array(OPTIONS_COUNT).fill(NOOP),
    );
  });

  it("scopes files with checked and allowed path patterns", () => {
    const code = ruleFixture("options.ts");
    assert.deepEqual(
      messages(code, RULE, { checkedPathPatterns: ["backend/**"] }, "backend/job.ts"),
      Array(OPTIONS_COUNT).fill(NOOP),
    );
    assert.deepEqual(
      messages(code, RULE, { checkedPathPatterns: ["backend/**"] }, "web/app.ts"),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        RULE,
        { checkedPathPatterns: ["backend/**"], allowedPathPatterns: ["backend/test/**"] },
        "backend/test/job.ts",
      ),
      [],
    );
    assert.deepEqual(messages(code, RULE, { allowedPathPatterns: ["web/**"] }, "web/app.ts"), []);
    assert.deepEqual(
      messages(
        code,
        RULE,
        { checkedPathPatterns: ["backend/**"] },
        resolve(process.cwd(), "backend/job.ts"),
      ),
      Array(OPTIONS_COUNT).fill(NOOP),
    );
  });

  it("skips originating callees that match allowedCalleeNamePatterns", () => {
    const code = ruleFixture("options.ts");
    assert.deepEqual(
      messages(code, RULE, { allowedCalleeNamePatterns: ["/^release/"] }, "options.ts"),
      [NOOP, NOOP],
    );
    assert.deepEqual(
      messages(code, RULE, { allowedCalleeNamePatterns: ["releaseLock"] }, "options.ts"),
      [NOOP, NOOP],
    );
  });

  it("ignores invalid regex patterns", () => {
    const code = ruleFixture("options.ts");
    assert.deepEqual(
      messages(code, RULE, { allowedCalleeNamePatterns: ["/[/"] }, "options.ts"),
      Array(OPTIONS_COUNT).fill(NOOP),
    );
    assert.deepEqual(messages(code, RULE, { checkedPathPatterns: ["/[/"] }, "backend/job.ts"), []);
    assert.deepEqual(
      messages(code, RULE, { checkedPathPatterns: ["backend/**", "/[/"] }, "backend/job.ts"),
      Array(OPTIONS_COUNT).fill(NOOP),
    );
  });
});
