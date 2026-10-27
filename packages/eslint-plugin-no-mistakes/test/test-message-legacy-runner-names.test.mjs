import { expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "test-no-error-message-matching";

it("snapshots test-message-legacy-runner-names diagnostics", async () => {
  const commonjsFile = "legacy-runner-commonjs.test.cjs";
  const cases = [
    {
      name: "keeps test/it checks without assuming renamed parameters are runners",
      file: "legacy-runner-names.test.ts",
      rule,
      findings: pluginFindings(
        fixture(`${rule}/legacy-runner-names.test.ts`),
        rule,
        undefined,
        "legacy-runner-names.test.ts",
      ),
    },
    {
      name: "preserves CommonJS module runner aliases",
      file: commonjsFile,
      rule,
      sourceType: "commonjs",
      findings: pluginFindings(
        fixture(`${rule}/${commonjsFile}`),
        rule,
        undefined,
        commonjsFile,
        {},
        "commonjs",
      ),
    },
    {
      name: "does not infer a namespace from repeated declarations",
      file: "legacy-runner-namespace-controls.test.js",
      rule,
      findings: pluginFindings(
        fixture(`${rule}/legacy-runner-namespace-controls.test.js`),
        rule,
        undefined,
        "legacy-runner-namespace-controls.test.js",
      ),
    },
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/test-message-legacy-runner-names.snap",
  );
});
