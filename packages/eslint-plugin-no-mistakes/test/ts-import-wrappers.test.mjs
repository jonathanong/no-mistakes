import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const fixtureRoot = resolve(
  __dirname,
  "../../../test-cases/eslint-snippets/fixture/ts-import-wrappers",
);

function source(file) {
  return readFileSync(resolve(fixtureRoot, file), "utf8");
}

function eslintCase(name, file, rule, options) {
  return {
    name,
    file,
    rule,
    ...(options === undefined ? {} : { options }),
    findings: pluginFindings(source(file), rule, options, resolve(fixtureRoot, file)),
  };
}

function oxlintOn(config, files) {
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(fixtureRoot, config),
      "--format",
      "json",
      ...files.map((file) => resolve(fixtureRoot, file)),
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.stdout.includes("Error running JS plugin"), false, result.stderr);
  const diagnostics = result.stdout ? (JSON.parse(result.stdout).diagnostics ?? []) : [];
  return {
    status: result.status,
    findings: files.map((file) => ({
      name: file,
      file,
      findings: oxlintFindings(
        diagnostics.filter((entry) => entry.filename?.endsWith(file)),
        file,
      ),
    })),
  };
}

describe("typescript import wrappers", () => {
  it("snapshots ESLint and Oxlint diagnostics", async () => {
    const cases = [
      eslintCase(
        "commonjs and import-equals use",
        "react-use-cjs.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "wrapped use(Promise.resolve)",
        "react-use-wrappers.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "unresolved react require",
        "react-use-unresolved.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "ignored unresolved react require",
        "react-use-unresolved.tsx",
        "react-no-use-promise-resolve",
        { unanalyzable: "ignore" },
      ),
      eslintCase(
        "import-equals and wrapped ReactNode",
        "react-nullish.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "unresolved ReactNode import",
        "react-nullish-unresolved.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "unresolved import type ReactNode",
        "react-nullish-import-type.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "ignored unresolved ReactNode import",
        "react-nullish-unresolved.tsx",
        "react-no-nullish-react-node",
        { unanalyzable: "ignore" },
      ),
      eslintCase("wrapped fetch method", "fetch-method.ts", "nextjs-static-fetch-method"),
      eslintCase("type-only fetch method", "fetch-type-only.ts", "nextjs-static-fetch-method"),
      eslintCase("wrapped fetch url", "fetch-url.ts", "nextjs-static-fetch-url"),
      eslintCase("type-only fetch url", "fetch-type-only.ts", "nextjs-static-fetch-url"),
      eslintCase("wrapped script attributes", "app/page.tsx", "nextjs-no-manual-script-tags", {
        allowInlineScriptIds: ["theme"],
      }),
      eslintCase("require next/head", "src/require-head.tsx", "nextjs-no-manual-script-tags"),
      eslintCase(
        "import-equals next/head",
        "src/import-equals-head.tsx",
        "nextjs-no-manual-script-tags",
      ),
      eslintCase("dynamic next require", "src/dynamic-next.tsx", "nextjs-no-manual-script-tags"),
      eslintCase(
        "ignored dynamic next require",
        "src/dynamic-next.tsx",
        "nextjs-no-manual-script-tags",
        { unanalyzable: "ignore" },
      ),
      eslintCase("static non-next require", "src/plain.tsx", "nextjs-no-manual-script-tags"),
      eslintCase(
        "commonjs nullable getter",
        "nullable-cjs.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "unresolved nullable getter",
        "nullable-unresolved.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "satisfies nullable getter",
        "nullable-satisfies.ts",
        "server-require-nullable-fetch-wrapper",
        { getterCalleePatterns: ["^api\\.getUser$"], requiredWrapperCallee: "withNullable" },
      ),
      eslintCase(
        "overloaded local callee",
        "nullable-overload.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase("wrapped jsx iife", "iife.tsx", "react-no-iife-in-jsx"),
      eslintCase(
        "nullable calls outside the getter",
        "nullable-outer.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "named unresolved ReactNode members",
        "react-nullish-members.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "local require is not React",
        "react-nullish-local-require.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "react use binding shapes",
        "react-use-bindings.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "promise type query alias",
        "promise-type-query.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "exempt scripts before an unresolved import",
        "src/script-exempt.tsx",
        "nextjs-no-manual-script-tags",
        {
          allowInlineScriptIds: ["theme"],
        },
      ),
      eslintCase(
        "structured JSON-LD stays uncertain",
        "src/script-exempt.tsx",
        "nextjs-no-manual-script-tags",
        { structuredDataComponent: "StructuredData", allowInlineScriptIds: ["theme"] },
      ),
      eslintCase(
        "type-only require is the loader",
        "react-use-type-require.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "value require shadows the loader",
        "react-use-value-require.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "annotated string is not a ReactNode",
        "react-nullish-annotated.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "member require getter",
        "nullable-member-require.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "local require is not the getter loader",
        "nullable-local-require.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "known member under an unresolved call",
        "nullable-known-member.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "dynamic require member stays unanalyzable",
        "nullable-dynamic-require.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "type-only import-equals is not next",
        "src/type-import-head.tsx",
        "nextjs-no-manual-script-tags",
      ),
      eslintCase(
        "type-only specifier is not next",
        "src/type-specifier-script.tsx",
        "nextjs-no-manual-script-tags",
      ),
      eslintCase(
        "mixed type and value next import",
        "src/mixed-script-import.tsx",
        "nextjs-no-manual-script-tags",
      ),
    ];

    expect(cases.find((item) => item.name === "ignored unresolved react require").findings).toEqual(
      [],
    );
    expect(
      cases.find((item) => item.name === "ignored unresolved ReactNode import").findings,
    ).toEqual([]);
    expect(cases.find((item) => item.name === "wrapped script attributes").findings).toEqual([]);
    expect(cases.find((item) => item.name === "ignored dynamic next require").findings).toEqual([]);
    expect(cases.find((item) => item.name === "static non-next require").findings).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "commonjs and import-equals use")
        .findings.map((f) => f.line),
    ).toEqual([3, 8, 13]);
    expect(
      cases
        .find((item) => item.name === "wrapped use(Promise.resolve)")
        .findings.map((f) => f.line),
    ).toEqual([3, 4, 5, 6]);
    expect(
      cases
        .find((item) => item.name === "import-equals and wrapped ReactNode")
        .findings.map((f) => f.line),
    ).toEqual([5, 9, 17, 21, 26]);
    expect(
      cases.find((item) => item.name === "wrapped fetch method").findings.map((f) => f.line),
    ).toEqual([2, 3, 4, 5]);
    expect(
      cases.find((item) => item.name === "wrapped fetch url").findings.map((f) => f.line),
    ).toEqual([5]);
    expect(
      cases.find((item) => item.name === "type-only fetch method").findings.map((f) => f.line),
    ).toEqual([4]);
    expect(
      cases.find((item) => item.name === "type-only fetch url").findings.map((f) => f.line),
    ).toEqual([5]);
    expect(
      cases.find((item) => item.name === "require next/head").findings.map((f) => f.messageId),
    ).toEqual(["script"]);
    expect(
      cases
        .find((item) => item.name === "import-equals next/head")
        .findings.map((f) => f.messageId),
    ).toEqual(["script"]);
    expect(
      cases.find((item) => item.name === "dynamic next require").findings.map((f) => f.messageId),
    ).toEqual(["unanalyzable"]);
    expect(
      cases.find((item) => item.name === "commonjs nullable getter").findings.map((f) => f.line),
    ).toEqual([3, 8, 13]);
    expect(
      cases.find((item) => item.name === "satisfies nullable getter").findings.map((f) => f.line),
    ).toEqual([5, 6, 8]);
    expect(cases.find((item) => item.name === "overloaded local callee").findings).toEqual([]);
    expect(
      cases.find((item) => item.name === "wrapped jsx iife").findings.map((f) => f.line),
    ).toEqual([3, 4, 5]);
    expect(
      cases.find((item) => item.name === "nullable calls outside the getter").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "named unresolved ReactNode members")
        .findings.map((finding) => finding.line),
    ).toEqual([7, 11]);
    expect(cases.find((item) => item.name === "local require is not React").findings).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "react use binding shapes")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [3, "resolve"],
      [15, "unanalyzable"],
      [22, "resolve"],
      [35, "resolve"],
    ]);
    expect(
      cases.find((item) => item.name === "promise type query alias").findings.map((f) => f.line),
    ).toEqual([4]);
    expect(
      cases
        .find((item) => item.name === "exempt scripts before an unresolved import")
        .findings.map((finding) => finding.line),
    ).toEqual([7]);
    expect(
      cases
        .find((item) => item.name === "structured JSON-LD stays uncertain")
        .findings.map((finding) => finding.line),
    ).toEqual([3, 7]);
    expect(
      cases
        .find((item) => item.name === "type-only require is the loader")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[4, "resolve"]]);
    expect(cases.find((item) => item.name === "value require shadows the loader").findings).toEqual(
      [],
    );
    expect(
      cases.find((item) => item.name === "annotated string is not a ReactNode").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "member require getter")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[2, "wrapper"]]);
    expect(
      cases.find((item) => item.name === "local require is not the getter loader").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "known member under an unresolved call")
        .findings.map((finding) => [finding.line, finding.column, finding.messageId]),
    ).toEqual([[4, 10, "unanalyzable"]]);
    expect(
      cases
        .find((item) => item.name === "dynamic require member stays unanalyzable")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [4, "unanalyzable"],
      [7, "unanalyzable"],
    ]);
    expect(
      cases.find((item) => item.name === "type-only import-equals is not next").findings,
    ).toEqual([]);
    expect(cases.find((item) => item.name === "type-only specifier is not next").findings).toEqual(
      [],
    );
    expect(
      cases
        .find((item) => item.name === "mixed type and value next import")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[2, "script"]]);

    const oxlint = {
      "react-use": oxlintOn(".oxlintrc-react-use.json", [
        "react-use-cjs.tsx",
        "react-use-wrappers.tsx",
        "react-use-unresolved.tsx",
        "react-use-bindings.tsx",
        "promise-type-query.tsx",
        "react-use-type-require.tsx",
        "react-use-value-require.tsx",
      ]),
      nullish: oxlintOn(".oxlintrc-nullish.json", [
        "react-nullish.tsx",
        "react-nullish-unresolved.tsx",
        "react-nullish-members.tsx",
        "react-nullish-local-require.tsx",
        "react-nullish-annotated.tsx",
      ]),
      "nullish-ignore": oxlintOn(".oxlintrc-nullish-ignore.json", ["react-nullish-unresolved.tsx"]),
      fetch: oxlintOn(".oxlintrc-fetch.json", [
        "fetch-method.ts",
        "fetch-url.ts",
        "fetch-type-only.ts",
      ]),
      script: oxlintOn(".oxlintrc-script.json", [
        "app/page.tsx",
        "src/require-head.tsx",
        "src/import-equals-head.tsx",
        "src/dynamic-next.tsx",
        "src/plain.tsx",
        "src/script-exempt.tsx",
        "src/type-import-head.tsx",
        "src/type-specifier-script.tsx",
        "src/mixed-script-import.tsx",
      ]),
      "script-structured": oxlintOn(".oxlintrc-script-structured.json", ["src/script-exempt.tsx"]),
      "script-ignore": oxlintOn(".oxlintrc-script-ignore.json", ["src/dynamic-next.tsx"]),
      "nullable-widget": oxlintOn(".oxlintrc-nullable-widget.json", [
        "nullable-cjs.ts",
        "nullable-unresolved.ts",
        "nullable-member-require.ts",
        "nullable-local-require.ts",
        "nullable-dynamic-require.ts",
        "nullable-known-member.ts",
      ]),
      "nullable-user": oxlintOn(".oxlintrc-nullable-user.json", ["nullable-satisfies.ts"]),
      "nullable-overload": oxlintOn(".oxlintrc-nullable-widget.json", ["nullable-overload.ts"]),
      "nullable-outer": oxlintOn(".oxlintrc-nullable-widget.json", ["nullable-outer.ts"]),
      iife: oxlintOn(".oxlintrc-iife.json", ["iife.tsx"]),
    };
    assert.equal(
      oxlint["nullish-ignore"].status,
      0,
      JSON.stringify(oxlint["nullish-ignore"].findings),
    );
    assert.equal(
      oxlint["script-ignore"].status,
      0,
      JSON.stringify(oxlint["script-ignore"].findings),
    );
    assert.equal(
      oxlint["nullable-overload"].status,
      0,
      JSON.stringify(oxlint["nullable-overload"].findings),
    );
    assert.equal(
      oxlint.script.findings.find((item) => item.file === "app/page.tsx").findings.length,
      0,
    );
    assert.equal(
      oxlint.script.findings.find((item) => item.file === "src/plain.tsx").findings.length,
      0,
    );

    await expect(
      snapshotDocument({
        eslint: snapshotCases(cases),
        oxlint: Object.fromEntries(
          Object.entries(oxlint).map(([name, result]) => [name, snapshotCases(result.findings)]),
        ),
      }),
    ).toMatchFileSnapshot("snapshots/ts-import-wrappers.snap");
  });
});
