import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../fixtures/eslint-plugin/upstreamed-generic", name),
    "utf8",
  );
}

describe("upstreamed generic rule scoping", () => {
  it("reports misplaced Next.js metadata exports only in Next.js paths", () => {
    assert.deepEqual(
      messages(
        fixture("metadata.invalid.ts"),
        "nextjs-metadata-exports-location",
        undefined,
        "app/lib/metadata.ts",
      ),
      ["location", "location", "location"],
    );
    assert.deepEqual(
      messages(
        fixture("metadata.invalid.ts"),
        "nextjs-metadata-exports-location",
        undefined,
        "lib/metadata.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("metadata.valid-page.tsx"),
        "nextjs-metadata-exports-location",
        undefined,
        "app/page.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("metadata.valid-page.tsx"),
        "nextjs-metadata-exports-location",
        undefined,
        "pages/page.tsx",
      ),
      ["location", "location"],
    );
    assert.deepEqual(
      messages(
        fixture("metadata.valid-page.tsx"),
        "nextjs-metadata-exports-location",
        undefined,
        "app/page.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("metadata.valid-page.tsx"),
        "nextjs-metadata-exports-location",
        undefined,
        "app/template.tsx",
      ),
      ["location", "location"],
    );
  });

  it("exempts scoped false positives", () => {
    assert.deepEqual(
      messages(
        fixture("placeholder-shadow.ts"),
        "no-placeholder-never-type-exports",
        undefined,
        "shadow.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("react-use-shadow.tsx"),
        "react-no-use-promise-resolve",
        undefined,
        "shadow.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("next-script-jsonld.tsx"),
        "nextjs-no-manual-script-tags",
        undefined,
        "app/page.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("next-script-inline-boot.tsx"),
        "nextjs-no-manual-script-tags",
        { allowInlineScriptIds: ["theme-init"] },
        "app/layout.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("next-script-inline-boot.tsx"),
        "nextjs-no-manual-script-tags",
        undefined,
        "app/layout.tsx",
      ),
      ["script"],
    );
    assert.deepEqual(
      messages(
        fixture("diagnostic-message.valid.test.ts"),
        "test-no-error-message-matching",
        undefined,
        "static-code-analysis/rules/example.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("generated-message.generated.test.ts"),
        "test-no-error-message-matching",
        undefined,
        "backend/services/email-address-validator/errors.generated.test.mts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("next-script-inline-boot.tsx"),
        "nextjs-no-manual-script-tags",
        { allowInlineScriptIdPatterns: ["^theme-"] },
        "app/layout.tsx",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        fixture("next-script-inline-boot.tsx"),
        "nextjs-no-manual-script-tags",
        { allowInlineScriptIdPatterns: ["["] },
        "app/layout.tsx",
      ),
      ["script"],
    );
  });

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
        fixture("shared-state-uncleaned-registry.invalid.test.ts"),
        "test-no-shared-state",
        undefined,
        "shared-state-uncleaned-registry.invalid.test.ts",
      ),
      ["shared"],
    );
  });
});
