import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/upstreamed-generic/fixture", name),
    "utf8",
  );
}

describe("shared-state callbacks", () => {
  it("avoids resolving shadowed non-function callbacks", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-callback-shadow.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-callback-shadow.valid.test.ts",
      ),
      [],
    );
  });

  it("respects shadowed local state in callbacks", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-shadowed-local-state.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-shadowed-local-state.valid.test.ts",
      ),
      [],
    );
  });
});
