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

describe("upstreamed generic cleanup registry scoping", () => {
  it("allows cleanup registries without allowing uncleaned shared state", () => {
    assert.deepEqual(
      messages(
        fixture("shared-state-cleanup-registry.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-cleanup-registry.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-cleanup-named.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-cleanup-named.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-nested-named-cleanup.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-nested-named-cleanup.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-sibling-named-cleanup.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-sibling-named-cleanup.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-uncleaned-registry.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-uncleaned-registry.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-mutating-hook.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-mutating-hook.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-once-hook.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-once-hook.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-nested-cleanup-helper.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-nested-cleanup-helper.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-non-reset-assignment.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-non-reset-assignment.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-out-of-scope-cleanup.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-out-of-scope-cleanup.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-cleanup-scope.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-cleanup-scope.invalid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-cleanup-path.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-cleanup-path.invalid.test.ts",
      ),
      ["shared", "shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-once-hook-allowed.valid.test.ts"),
        "test-no-shared-state",
        { allowBeforeAllAssignments: true },
        "shared-state-once-hook-allowed.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-once-hook-allowed.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-once-hook-allowed.valid.test.ts",
      ),
      ["shared"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-test-beforeall.valid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-test-beforeall.valid.test.ts",
      ),
      [],
      "test.beforeAll member-access form should not flag assignments",
    );
  });
});
