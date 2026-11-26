import { expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";
import { ruleFixture } from "./require-options-on-imported-call-shared.mjs";

const fetchFile = "nextjs-static-fetch-url/opaque-arguments.ts";
const methodFile = "nextjs-static-fetch-method/opaque-properties.ts";
const localeFile = "no-locale-less-date-format/stable-aliases.ts";
const scopeFile = "no-locale-less-date-format/dynamic-scope-aliases.js";
const optionsFile = "require-options-on-imported-call/opaque-arguments.ts";
const constructors = ["Intl.DateTimeFormat"];
const targets = [
  {
    sourceSpecifierPatterns: ["@fixture/jobs"],
    calleeNamePatterns: ["enqueue"],
    optionsPosition: 2,
    requiredProperties: ["timeoutMs"],
  },
];

it("snapshots opaque arguments, fetch option precedence, and stable locale aliases", async () => {
  const calls = [
    ["nextjs-static-fetch-url", fetchFile, fixture(fetchFile)],
    ["nextjs-static-fetch-method", fetchFile, fixture(fetchFile)],
    ["nextjs-static-fetch-method", methodFile, fixture(methodFile)],
    [
      "no-locale-less-date-format",
      localeFile,
      fixture(localeFile),
      {
        constructors,
        unknownValues: "finding",
      },
    ],
    ["no-locale-less-date-format", localeFile, fixture(localeFile), { constructors }],
    [
      "no-locale-less-date-format",
      scopeFile,
      fixture(scopeFile),
      {
        constructors,
        unknownValues: "finding",
      },
      "script",
    ],
    ["no-locale-less-date-format", scopeFile, fixture(scopeFile), { constructors }, "script"],
    [
      "require-options-on-imported-call",
      optionsFile,
      ruleFixture("opaque-arguments.ts"),
      { targets },
    ],
  ];
  const cases = calls.map(([rule, file, code, options, sourceType]) => ({
    name: `${file.split("/").at(-1)}: ${options?.unknownValues ?? "default"}`,
    rule,
    file,
    options,
    findings: pluginFindings(code, rule, options, file, {}, sourceType),
  }));
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/opaque-rule-inputs.snap",
  );
});
