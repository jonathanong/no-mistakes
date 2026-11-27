import { expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "no-placeholder-never-type-exports";
const file = "ts-object-recall/placeholder-types.ts";

it("reports unanalyzable type re-exports and keeps a same-file never export", async () => {
  const findings = pluginFindings(fixture(file), rule, undefined, file);
  expect(findings.map((finding) => [finding.line, finding.messageId])).toEqual([
    [1, "placeholder"],
    [3, "placeholder"],
    [6, "unanalyzable"],
    [7, "unanalyzable"],
    [8, "unanalyzable"],
    [11, "unanalyzable"],
    [15, "placeholder"],
  ]);
  await expect(
    snapshotDocument(
      snapshotCases([
        {
          name: "same-file never stays a placeholder and unread re-exports are unanalyzable",
          file,
          findings,
          rule,
        },
      ]),
    ),
  ).toMatchFileSnapshot("snapshots/no-placeholder-never-type-exports.snap");
});
