import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

function ruleFixture(rule, name) {
  return readFileSync(resolve(__dirname, "../../../fixtures/eslint-plugin", rule, name), "utf8");
}

describe("ts-no-export-renaming", () => {
  it("allows direct value exports and type-only aliases", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-export-renaming", "valid.ts"),
        "ts-no-export-renaming",
        undefined,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports value export aliases", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-export-renaming", "invalid.ts"),
        "ts-no-export-renaming",
        undefined,
        "invalid.ts",
      ),
      ["renamed", "renamed", "renamed"],
    );
  });

  it("covers direct exports, string-literal export names, and empty export lists", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-export-renaming", "coverage.ts"),
        "ts-no-export-renaming",
        undefined,
        "coverage.ts",
      ),
      ["renamed", "renamed"],
    );
  });

  it("supports default re-export and path scoping options", () => {
    const code = ruleFixture("ts-no-export-renaming", "options.ts");
    assert.deepEqual(messages(code, "ts-no-export-renaming", undefined, "web/app/index.ts"), [
      "renamed",
      "renamed",
    ]);
    assert.deepEqual(
      messages(code, "ts-no-export-renaming", { allowDefaultReExports: true }, "web/app/index.ts"),
      ["renamed"],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-no-export-renaming",
        { includePathPatterns: ["^backend/"] },
        resolve(__dirname, "../web/app/index.ts"),
      ),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        "ts-no-export-renaming",
        { includePathPatterns: ["^backend/", "["] },
        resolve(__dirname, "../backend/index.ts"),
      ),
      ["renamed", "renamed"],
    );
  });
});

describe("ts-no-function-aliases", () => {
  it("allows wrappers with behavior beyond direct forwarding", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-function-aliases", "valid.ts"),
        "ts-no-function-aliases",
        undefined,
        "valid.ts",
      ),
      [],
    );
  });

  it("reports simple wrappers that only forward to another function", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-function-aliases", "invalid.ts"),
        "ts-no-function-aliases",
        undefined,
        "invalid.ts",
      ),
      ["alias", "alias", "alias", "alias", "alias", "alias", "alias"],
    );
  });

  it("covers function expressions, self calls, default params, and TS expression wrappers", () => {
    assert.deepEqual(
      messages(
        ruleFixture("ts-no-function-aliases", "coverage.ts"),
        "ts-no-function-aliases",
        undefined,
        "coverage.ts",
      ),
      ["alias", "alias", "alias"],
    );
  });
});

describe("react-no-nullish-react-node", () => {
  it("allows explicit undefined checks and non-ReactNode nullish expressions", () => {
    assert.deepEqual(
      messages(
        ruleFixture("react-no-nullish-react-node", "valid.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "valid.tsx",
      ),
      [],
    );
  });

  it("reports nullish coalescing on explicitly typed ReactNode values", () => {
    assert.deepEqual(
      messages(
        ruleFixture("react-no-nullish-react-node", "invalid.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "invalid.tsx",
      ),
      ["nullish", "nullish", "nullish"],
    );
  });

  it("covers ReactNode aliases, typed variables, function expressions, and type literal props", () => {
    assert.deepEqual(
      messages(
        ruleFixture("react-no-nullish-react-node", "coverage.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "coverage.tsx",
      ),
      [
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
        "nullish",
      ],
    );
  });
});
