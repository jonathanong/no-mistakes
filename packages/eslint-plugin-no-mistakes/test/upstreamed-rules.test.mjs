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

describe("upstreamed generic rules", () => {
  it("passes clean generic examples", () => {
    const code = fixture("valid.tsx");
    const checks = [
      ["await-array-methods", "valid.tsx"],
      ["no-delete-property", "valid.tsx"],
      ["no-placeholder-never-type-exports", "valid.tsx"],
      ["test-no-shared-state", "valid.tsx"],
      ["test-no-error-message-matching", "valid.tsx"],
      ["no-vitest-sequential", "valid.tsx"],
      ["react-no-use-promise-resolve", "valid.tsx"],
      ["react-no-iife-in-jsx", "valid.tsx"],
      ["nextjs-no-manual-script-tags", "valid.tsx"],
      ["nextjs-metadata-exports-location", "app/page.tsx"],
    ];

    for (const [rule, filename] of checks) {
      assert.deepEqual(messages(code, rule, undefined, filename), [], rule);
    }
  });

  it("reports generic invalid examples", () => {
    const code = fixture("invalid.tsx");
    const expected = [
      ["await-array-methods", ["awaited", "awaited", "awaited"]],
      ["no-delete-property", ["delete"]],
      ["no-placeholder-never-type-exports", ["placeholder", "placeholder", "placeholder"]],
      ["test-no-shared-state", ["shared"]],
      ["test-no-error-message-matching", ["message", "message", "message"]],
      ["no-vitest-sequential", ["sequential"]],
      ["react-no-use-promise-resolve", ["resolve", "resolve", "resolve", "resolve", "resolve"]],
      ["react-no-iife-in-jsx", ["iife", "iife"]],
    ];

    for (const [rule, ids] of expected) {
      assert.deepEqual(messages(code, rule, undefined, "invalid.tsx"), ids, rule);
    }
    assert.deepEqual(messages(code, "nextjs-no-manual-script-tags", undefined, "app/page.tsx"), [
      "script",
    ]);
    assert.deepEqual(
      messages(code, "nextjs-no-manual-script-tags", undefined, "components/widget.tsx"),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("next-script-import.tsx"),
        "nextjs-no-manual-script-tags",
        undefined,
        "components/widget.tsx",
      ),
      ["script"],
    );
    assert.deepEqual(
      messages(
        fixture("shared-state-shadow.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-shadow.invalid.test.ts",
      ),
      ["shared"],
    );
  });

  it("reports import-only test aggregators", () => {
    assert.deepEqual(
      messages(
        fixture("import-only.invalid.test.ts"),
        "no-import-only-test-files",
        undefined,
        "import-only.invalid.test.ts",
      ),
      ["aggregate"],
    );
    assert.deepEqual(
      messages(
        fixture("import-only.valid.test.ts"),
        "no-import-only-test-files",
        undefined,
        "import-only.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("import-only.invalid.test.cjs"),
        "no-import-only-test-files",
        undefined,
        "import-only.invalid.test.cjs",
      ),
      ["aggregate"],
    );
  });

  it("tracks JSX IIFE invocations inside expression arrays", () => {
    const code = `
      export default function Component() {
        return <>{[ (function renderTag() { return "tag"; })() ]}</>;
      }
    `;
    assert.deepEqual(messages(code, "react-no-iife-in-jsx"), ["iife"]);
  });

  it("reports mock test filename mismatches", () => {
    assert.deepEqual(
      messages(
        fixture("mock-name.invalid.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.invalid.test.ts",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.invalid.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.invalid.mock.test.ts",
      ),
      ["unnecessaryMock"],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.valid.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.valid.mock.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.valid.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.valid.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.global.invalid.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.global.invalid.test.ts",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.cjs.invalid.test.cjs"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.cjs.invalid.test.cjs",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.fn-stub.valid.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.fn-stub.valid.test.ts",
      ),
      [],
      "typed vi.fn<T>() callback stub with no module mocking should not require .mock.test (#388)",
    );
    assert.deepEqual(
      messages(
        fixture("mock-name.fn-stub.invalid.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "mock-name.fn-stub.invalid.mock.test.ts",
      ),
      ["unnecessaryMock"],
      "vi.fn() alone does not justify a .mock.test filename (#388)",
    );
  });

  it("reports Playwright policy violations", () => {
    const code = fixture("playwright.invalid.ts");
    assert.deepEqual(messages(code, "playwright-assertion-timeout-cap", undefined, "e2e.spec.ts"), [
      "timeout",
    ]);
    assert.deepEqual(messages(code, "playwright-selector-priority", undefined, "e2e.spec.ts"), [
      "semantic",
      "heading",
      "text",
    ]);
    assert.deepEqual(messages(code, "playwright-no-set-timeout", undefined, "e2e.spec.ts"), [
      "timeout",
      "timeout",
      "timeout",
    ]);

    const valid = fixture("playwright.valid.ts");
    assert.deepEqual(
      messages(valid, "playwright-assertion-timeout-cap", undefined, "e2e.spec.ts"),
      [],
    );
    assert.deepEqual(messages(valid, "playwright-selector-priority", undefined, "e2e.spec.ts"), []);
    assert.deepEqual(messages(valid, "playwright-no-set-timeout", undefined, "e2e.spec.ts"), []);
    assert.deepEqual(
      messages(
        fixture("playwright.non-test.ts"),
        "playwright-no-set-timeout",
        undefined,
        "playwright.non-test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("playwright.non-test.ts"),
        "playwright-no-set-timeout",
        undefined,
        "app/timer.ts",
      ),
      [],
    );
  });
});
