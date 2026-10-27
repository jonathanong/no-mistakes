import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import {
  configuredFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { fixture, messages } from "./helpers.mjs";

const snippet = (name) => fixture(name);

describe("test and react same-file recall", () => {
  it("reports vitest sequential through aliases and static keys", () => {
    assert.equal(
      messages(
        snippet("no-vitest-sequential/recall.test.ts"),
        "no-vitest-sequential",
        undefined,
        "recall.test.ts",
      ).length,
      15,
    );
    assert.deepEqual(
      messages(
        snippet("no-vitest-sequential/created-runner.test.ts"),
        "no-vitest-sequential",
        undefined,
        "created-runner.test.ts",
      ),
      ["sequential"],
    );
    for (const name of [
      "renamed-fixture.test.ts",
      "default-import.test.ts",
      "module-function.test.ts",
      "module-let.test.ts",
      "module-var.test.ts",
    ]) {
      assert.deepEqual(
        messages(snippet(`no-vitest-sequential/${name}`), "no-vitest-sequential", undefined, name),
        ["sequential"],
      );
    }
    assert.deepEqual(
      messages(
        snippet("no-vitest-sequential/const-alias.test.ts"),
        "no-vitest-sequential",
        undefined,
        "const-alias.test.ts",
      ),
      ["sequential", "sequential"],
    );
  });

  it("treats an expression-free template require as a test import", () => {
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/recall.test.js"),
        "no-import-only-test-files",
        undefined,
        "recall.test.js",
      ),
      ["aggregate"],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/dynamic.test.js"),
        "no-import-only-test-files",
        undefined,
        "dynamic.test.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/suppressed.test.js"),
        "no-import-only-test-files",
        undefined,
        "suppressed.test.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/forwarded.test.js"),
        "no-import-only-test-files",
        undefined,
        "forwarded.test.js",
      ),
      ["aggregate"],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/dynamic-forward.test.js"),
        "no-import-only-test-files",
        undefined,
        "dynamic-forward.test.js",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        snippet("no-import-only-test-files/next-line.test.js"),
        "no-import-only-test-files",
        undefined,
        "next-line.test.js",
      ),
      [],
    );
  });

  it("recognizes vitest namespace module mocks and ignores the default export", () => {
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/recall.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "recall.test.ts",
      ),
      ["needsMock"],
    );
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/default.mock.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "default.mock.test.ts",
      ),
      ["unnecessaryMock"],
    );
    assert.deepEqual(
      messages(
        snippet("vitest-mock-test-file-naming/suppressed.test.ts"),
        "vitest-mock-test-file-naming",
        undefined,
        "suppressed.test.ts",
      ),
      [],
    );
  });

  it("snapshots test diagnostics", async () => {
    const groups = [
      [
        "test-no-error-message-matching",
        [
          "recall.test.ts",
          "playwright-expect.test.ts",
          "setup-expect.test.ts",
          "bun-expect.test.ts",
          "chai-expect.test.ts",
          "cjs-expect.test.ts",
          "local-expect.test.ts",
          "function-expect.test.ts",
          "let-expect.test.ts",
          "bracket-call.test.ts",
          "binary-suppressed.test.ts",
          "matcher-call.test.ts",
        ],
      ],
      [
        "test-no-shared-state",
        [
          "recall.test.ts",
          "computed-describe.test.ts",
          "alias-describe.test.ts",
          "sequence-describe.test.ts",
          "serial-alias.test.ts",
          "shadow-test-alias.test.ts",
          "namespace-serial.test.ts",
          "suppressed-alias.test.ts",
        ],
      ],
      ["test-no-delayed-rejects", ["recall.ts", "poll.ts", "suppressed.ts"]],
    ];
    const cases = groups.flatMap(([rule, files]) =>
      files.map((file) => ({
        file,
        findings: pluginFindings(snippet(`${rule}/${file}`), rule, undefined, file),
        name: `${rule} ${file}`,
        rule,
      })),
    );
    cases.push(
      {
        file: "commonjs.test.cjs",
        findings: pluginFindings(
          snippet("test-no-error-message-matching/commonjs.test.cjs"),
          "test-no-error-message-matching",
          undefined,
          "commonjs.test.cjs",
          {},
          "commonjs",
        ),
        name: "test-no-error-message-matching commonjs.test.cjs",
        rule: "test-no-error-message-matching",
        sourceType: "commonjs",
      },
      {
        file: "shadow-namespace.test.ts",
        findings: configuredFindings(
          snippet("test-no-shared-state/shadow-namespace.test.ts"),
          { "no-mistakes/test-no-shared-state": "error" },
          "shadow-namespace.test.ts",
        ),
        name: "test-no-shared-state shadow-namespace.test.ts",
        rule: "test-no-shared-state",
      },
    );
    await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
      "snapshots/recall-test-react.snap",
    );
  });
});
