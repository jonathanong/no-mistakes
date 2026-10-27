import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/upstreamed-generic/fixture", name),
    "utf8",
  );
}

describe("upstreamed generic cleanup registry scoping", () => {
  it("snapshots test-no-shared-state diagnostics", async () => {
    const specs = [
      ["shared-state-cleanup-registry.valid.test.ts"],
      ["shared-state-cleanup-named.valid.test.ts"],
      ["shared-state-nested-named-cleanup.valid.test.ts"],
      ["shared-state-sibling-named-cleanup.valid.test.ts"],
      ["shared-state-uncleaned-registry.invalid.test.ts"],
      ["shared-state-mutating-hook.invalid.test.ts"],
      ["shared-state-once-hook.invalid.test.ts"],
      ["shared-state-nested-cleanup-helper.invalid.test.ts"],
      ["shared-state-non-reset-assignment.invalid.test.ts"],
      ["shared-state-out-of-scope-cleanup.invalid.test.ts"],
      ["shared-state-cleanup-scope.invalid.test.ts"],
      ["shared-state-cleanup-path.invalid.test.ts"],
      ["shared-state-once-hook-allowed.valid.test.ts", { allowBeforeAllAssignments: true }],
      ["shared-state-once-hook-allowed.valid.test.ts"],
      ["shared-state-test-beforeall.valid.test.ts"],
    ];
    const cases = specs.map(([file, options]) => ({
      file,
      findings: pluginFindings(fixture(file), "test-no-shared-state", options, file),
      name: options === undefined ? file : `${file} ${JSON.stringify(options)}`,
      rule: "test-no-shared-state",
      ...(options === undefined ? {} : { options }),
    }));
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/upstreamed-rules-cleanup.snap",
    );
  });
});
