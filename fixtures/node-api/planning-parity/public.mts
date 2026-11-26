import {
  analyzeProject,
  impactedChecks,
  testsPlan,
  type GitDiffRevisions,
} from "../../../packages/no-mistakes/index.js";

const revisions: GitDiffRevisions = { base: "HEAD~1", head: "HEAD" };
void impactedChecks({ ...revisions, diffFile: "change.patch" });
void impactedChecks({ diff: "diff --git a/example.ts b/example.ts" });
void testsPlan({ ...revisions, framework: "vitest" });
void testsPlan({ framework: "vitest", fromGitDiff: "HEAD~1...HEAD" });
void analyzeProject({ reports: [{ type: "impactedChecks", ...revisions }] });
// @ts-expect-error A head revision needs a base.
void impactedChecks({ head: "HEAD" });
// @ts-expect-error A head revision needs a base.
void testsPlan({ framework: "vitest", head: "HEAD" });
// @ts-expect-error Batched planning also needs a base when head is present.
void analyzeProject({ reports: [{ type: "testsPlan", head: "HEAD" }] });
