import assert from "node:assert/strict";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { lint, plugin } from "./helpers.mjs";
import {
  headingSection,
  knownRecallFailures,
  listedDocFiles,
  ruleOption,
  rulePages,
  snippetFilename,
  sourceFences,
  staleKnownRecallFailures,
} from "./docs-snippets.mjs";

describe("known recall failures stay listed only while they reproduce", () => {
  it("treats a vanished miss as stale", () => {
    assert.deepEqual(
      staleKnownRecallFailures(["Disallowed ts hits=0 ids="], ["Disallowed ts hits=0 ids="]),
      [],
    );
    assert.deepEqual(staleKnownRecallFailures([], ["Disallowed ts hits=0 ids="]), [
      "Disallowed ts hits=0 ids=",
    ]);
  });
});

describe("eslint rule docs are executable", () => {
  it("documents every exported rule", () => {
    const files = listedDocFiles();
    const expected = Object.keys(plugin.rules)
      .map((id) => `${id}.md`)
      .sort();
    assert.deepEqual(files, expected);
  });

  for (const page of rulePages()) {
    it(`${page.ruleId} disallowed reports and allowed stays silent`, () => {
      const disallowed = sourceFences(headingSection(page.body, "Disallowed"));
      const allowed = sourceFences(headingSection(page.body, "Allowed"));
      assert.notEqual(disallowed.length, 0, `${page.file} needs a Disallowed source fence`);
      assert.notEqual(allowed.length, 0, `${page.file} needs an Allowed source fence`);

      const option = ruleOption(headingSection(page.body, "Options"), page.ruleId);
      const failures = [];
      for (const [kind, expectReport] of [
        [disallowed, true],
        [allowed, false],
      ]) {
        for (const snippet of kind) {
          const result = runSnippet(page.ruleId, snippet, option);
          if (result.parseError) {
            failures.push(`${expectReport ? "Disallowed" : "Allowed"} parse: ${result.parseError}`);
            continue;
          }
          const reported = result.hits > 0;
          if (reported !== expectReport) {
            failures.push(
              `${expectReport ? "Disallowed" : "Allowed"} ${snippet.lang} hits=${result.hits} ids=${result.ids}`,
            );
          }
        }
      }

      const known = knownRecallFailures.get(page.ruleId) ?? [];
      assert.deepEqual(
        failures.filter((failure) => !known.includes(failure)),
        [],
      );
      assert.deepEqual(
        staleKnownRecallFailures(failures, known),
        [],
        `${page.ruleId} knownRecallFailures must still reproduce`,
      );
    });
  }

  it("await-array-methods suppression hides a real documented finding", () => {
    const page = rulePages().find((candidate) => candidate.ruleId === "await-array-methods");
    const [snippet] = sourceFences(headingSection(page.body, "Suppression"));
    assert.ok(snippet, "await-array-methods needs a Suppression source fence");

    const directive =
      /^\s*\/\/ eslint-disable-next-line no-mistakes\/await-array-methods --[^\n]*\r?\n/m;
    const unsuppressedCode = snippet.code.replace(directive, "");
    assert.notEqual(unsuppressedCode, snippet.code, "the snippet needs its rule directive");
    assert.match(unsuppressedCode, /await values\.sort\(\);/);
  });

  it("snapshots await-array-methods suppression diagnostics", async () => {
    const page = rulePages().find((candidate) => candidate.ruleId === "await-array-methods");
    const [snippet] = sourceFences(headingSection(page.body, "Suppression"));
    const filename = snippetFilename(snippet.code, snippet.lang);
    const directive =
      /^\s*\/\/ eslint-disable-next-line no-mistakes\/await-array-methods --[^\n]*\r?\n/m;
    const unsuppressedCode = snippet.code.replace(directive, "");
    const typescript = {
      parser: tsParser,
      sourceType: "module",
      globals: {},
    };
    const languageOptions = {
      ecmaVersion: 2024,
      sourceType: "module",
      globals: {},
      parserOptions: { ecmaFeatures: { jsx: true } },
    };
    await expect(
      snapshotDocument({
        "await-array-methods": snapshotSuite("await-array-methods", languageOptions, {
          valid: [{ code: snippet.code, filename, languageOptions: typescript }],
          invalid: [
            // 1 awaited diagnostic remains after the suppression directive is removed.
            { code: unsuppressedCode, filename, languageOptions: typescript },
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/docs-snippets-remaining.snap");
  }, 30_000);
});

function runSnippet(ruleId, snippet, option) {
  const filename = snippetFilename(snippet.code, snippet.lang);
  const rule = option === undefined ? "error" : ["error", option];
  try {
    const messages = lint(snippet.code, { [`no-mistakes/${ruleId}`]: rule }, filename);
    const parseError = messages.find((message) => message.fatal || !message.ruleId);
    const hits = messages.filter((message) => message.ruleId === `no-mistakes/${ruleId}`);
    return {
      parseError: parseError?.message,
      hits: hits.length,
      ids: hits.map((message) => message.messageId).join(","),
    };
  } catch (error) {
    return { parseError: String(error.message ?? error), hits: 0, ids: "" };
  }
}
