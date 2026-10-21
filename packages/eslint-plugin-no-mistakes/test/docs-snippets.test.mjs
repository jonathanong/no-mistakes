import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { lint, plugin } from "./helpers.mjs";
import {
  headingSection,
  knownRecallFailures,
  listedDocFiles,
  ruleOption,
  rulePages,
  snippetFilename,
  sourceFences,
} from "./docs-snippets.mjs";

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
    });
  }
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
