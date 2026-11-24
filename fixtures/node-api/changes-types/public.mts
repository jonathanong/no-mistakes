import { changesDiff, testsPlan } from "../../../packages/no-mistakes/index.js";
import type { ChangesConfig, ChangesInputReport, ChangesReport } from "../../../packages/no-mistakes/index.js";

const config: ChangesConfig = {
  categories: {
    fixtures: {
      paths: ["fixtures/**/*.json"],
      id: { keysOf: { format: "json", array: "fixtures", idField: "id" } },
      tests: ["tests/**/*.test.ts"],
      match: 'load("{id}")',
      fallback: "full-suite",
    },
  },
};
void config;
const input: ChangesInputReport<"fixtures"> = {
  schemaVersion: 1, complete: true, unclassified: [], fixtures: { changed: ["example"] },
};
void input;
const pending: Promise<ChangesReport<"fixtures">> = changesDiff<"fixtures">({ baseRoot: "before", headRoot: "after" });
const report = await pending;
const ids: string[] | undefined = report.fixtures?.changed;
void ids;
void testsPlan({ root: "after", changes: "changes.json", framework: "vitest" });
// @ts-expect-error Directory roots are required.
void changesDiff({ headRoot: "after" });
// @ts-expect-error Direct-owner mode cannot consume semantic policy inputs.
void testsPlan({ framework: "vitest", directTestOwner: true, changes: "changes.json" });
