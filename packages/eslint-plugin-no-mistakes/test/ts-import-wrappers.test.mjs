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
        "reassigned alias chain stays bounded",
        "react-use-alias-chain.tsx",
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
        "inner type require does not hide an outer parameter",
        "react-use-inner-type-require.tsx",
        "react-no-use-promise-resolve",
      ),
      eslintCase(
        "import binding edges stay on their own write",
        "react-use-binding-edges.tsx",
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
      eslintCase(
        "escaped next template import",
        "src/escaped-script-import.tsx",
        "nextjs-no-manual-script-tags",
      ),
      eslintCase(
        "unresolved relay around a getter",
        "nullable-relay-getter.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "ignored unresolved relay around a getter",
        "nullable-relay-getter.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
          unanalyzable: "ignore",
        },
      ),
      eslintCase(
        "JSON-LD type before a spread still reports",
        "app/script-spread.tsx",
        "nextjs-no-manual-script-tags",
      ),
      eslintCase(
        "member alias of a static require getter",
        "nullable-member-alias.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "forwarded member of an unresolved require",
        "nullable-forwarded-member.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "destructured name from a dynamic require",
        "nullable-unresolved-name.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "ignored destructured name from a dynamic require",
        "nullable-unresolved-name.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
          unanalyzable: "ignore",
        },
      ),
      eslintCase(
        "const alias chain keeps an unresolved ReactNode",
        "react-nullish-alias-chain.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "pattern shadows do not keep an outer ReactNode",
        "react-nullish-pattern-shadows.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "assertions classify the asserted type",
        "react-nullish-assertions.ts",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "named default import is the React namespace",
        "react-nullish-named-default.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "let require registers a React namespace",
        "react-nullish-let-require.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "computed type keys are static literals only",
        "react-nullish-computed-props.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "erased require and a later assignment",
        "nullable-assigned-require.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "var require and inline require calls",
        "nullable-binding-shapes.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "alternation does not prove a non-getter",
        "nullable-alternation.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^api\\.(getWidget|getGadget)$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "nested type and catch parameter shadow outer ReactNode",
        "react-nullish-nested-shadow.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "merged interfaces and nested extends",
        "react-nullish-merged.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "loop const and class shadow an outer binding",
        "react-nullish-loop-class.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "local merged interface and alias cycle",
        "react-nullish-local-cycle.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "direct type alias cycle reaches the cycle guard",
        "react-nullish-self-alias.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "nested heritage uses the nearest declaration",
        "react-nullish-nested-heritage.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "mixed nested heritage stays unfinished",
        "react-nullish-mixed-heritage.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "shadowed ReactNode bindings stay on their own declaration",
        "react-nullish-binding-shadows.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "scope edges keep each ReactNode binding",
        "react-nullish-scope-edges.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "numeric literal keys stay on that property",
        "react-nullish-numeric-keys.tsx",
        "react-no-nullish-react-node",
      ),
      eslintCase(
        "unresolved qualified getter terminal",
        "nullable-qualified-member.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^api\\.getWidget$", "^services\\.api\\.getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "named import member chain",
        "nullable-named-chain.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^api\\.getWidget$", "^services\\.api\\.getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "nested destructure keeps the member path",
        "nullable-nested-destructure.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^api\\.getWidget$", "^services\\.api\\.getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "default member alias keeps its prefix",
        "nullable-default-alias.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^default\\.getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "computed member of a namespace import",
        "nullable-computed-member.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "unresolved wrapper is not inside",
        "nullable-unresolved-wrapper.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
        },
      ),
      eslintCase(
        "ignored unresolved wrapper",
        "nullable-unresolved-wrapper.ts",
        "server-require-nullable-fetch-wrapper",
        {
          getterCalleePatterns: ["^getWidget$"],
          requiredWrapperCallee: "wrapNullable",
          nullableReturnTypeNames: ["Widget"],
          unanalyzable: "ignore",
        },
      ),
    ];

    expect(cases.find((item) => item.name === "ignored unresolved react require").findings).toEqual(
      [],
    );
    expect(
      cases.find((item) => item.name === "ignored unresolved ReactNode import").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "unresolved ReactNode import")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "unanalyzable"],
      [13, "unanalyzable"],
      [23, "unanalyzable"],
      [33, "unanalyzable"],
      [38, "unanalyzable"],
    ]);
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
    ).toEqual([5, 9, 17, 21, 26, 40]);
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
    ).toEqual([5, 7]);
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
      [43, "resolve"],
      [48, "resolve"],
      [53, "resolve"],
      [58, "resolve"],
      [64, "unanalyzable"],
      [76, "unanalyzable"],
      [83, "unanalyzable"],
      [86, "unanalyzable"],
      [110, "resolve"],
      [115, "unanalyzable"],
      [118, "resolve"],
      [122, "resolve"],
      [126, "resolve"],
      [132, "unanalyzable"],
      [147, "resolve"],
      [152, "unanalyzable"],
      [155, "resolve"],
      [166, "unanalyzable"],
      [170, "unanalyzable"],
      [179, "resolve"],
      [184, "resolve"],
    ]);
    expect(
      cases.find((item) => item.name === "promise type query alias").findings.map((f) => f.line),
    ).toEqual([4, 9]);
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
      cases
        .find((item) => item.name === "inner type require does not hide an outer parameter")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[4, "resolve"]]);
    expect(
      cases
        .find((item) => item.name === "import binding edges stay on their own write")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [7, "unanalyzable"],
      [12, "unanalyzable"],
      [17, "resolve"],
      [24, "resolve"],
      [32, "unanalyzable"],
      [52, "resolve"],
      [58, "resolve"],
      [64, "unanalyzable"],
    ]);
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
      [10, "unanalyzable"],
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
    expect(
      cases
        .find((item) => item.name === "escaped next template import")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[2, "script"]]);
    expect(
      cases
        .find((item) => item.name === "unresolved relay around a getter")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [4, "unanalyzable"],
      [4, "unanalyzable"],
    ]);
    expect(
      cases.find((item) => item.name === "ignored unresolved relay around a getter").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "JSON-LD type before a spread still reports")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "script"],
      [7, "script"],
    ]);
    expect(
      cases
        .find((item) => item.name === "member alias of a static require getter")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [6, "wrapper"],
      [10, "unanalyzable"],
      [18, "wrapper"],
      [22, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "forwarded member of an unresolved require")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "unanalyzable"],
      [9, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "destructured name from a dynamic require")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[5, "unanalyzable"]]);
    expect(
      cases.find((item) => item.name === "ignored destructured name from a dynamic require")
        .findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "const alias chain keeps an unresolved ReactNode")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [17, "nullish"],
      [23, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "pattern shadows do not keep an outer ReactNode")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([[34, "nullish"]]);
    expect(
      cases
        .find((item) => item.name === "assertions classify the asserted type")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [10, "nullish"],
      [15, "nullish"],
      [23, "nullish"],
      [27, "nullish"],
      [32, "nullish"],
      [51, "nullish"],
      [71, "unanalyzable"],
      [75, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "named default import is the React namespace")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "nullish"],
      [9, "nullish"],
    ]);
    expect(
      cases
        .find((item) => item.name === "let require registers a React namespace")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "nullish"],
      [10, "unanalyzable"],
      [16, "unanalyzable"],
      [21, "nullish"],
      [26, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "computed type keys are static literals only")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [10, "nullish"],
      [23, "nullish"],
    ]);
    expect(
      cases
        .find((item) => item.name === "var require and inline require calls")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [6, "wrapper"],
      [13, "unanalyzable"],
      [17, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "alternation does not prove a non-getter")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "unanalyzable"],
      [10, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "nested type and catch parameter shadow outer ReactNode")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [9, "unanalyzable"],
      [22, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "merged interfaces and nested extends")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [14, "nullish"],
      [24, "unanalyzable"],
      [35, "nullish"],
      [43, "unanalyzable"],
      [46, "unanalyzable"],
      [55, "nullish"],
      [75, "nullish"],
    ]);
    expect(
      cases
        .find((item) => item.name === "loop const and class shadow an outer binding")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [6, "unanalyzable"],
      [19, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "local merged interface and alias cycle")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [15, "nullish"],
      [37, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "direct type alias cycle reaches the cycle guard")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [3, "unanalyzable"],
      [8, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "nested heritage uses the nearest declaration")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [15, "nullish"],
      [26, "unanalyzable"],
      [35, "nullish"],
    ]);
    expect(
      cases.find((item) => item.name === "mixed nested heritage stays unfinished").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "shadowed ReactNode bindings stay on their own declaration")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [16, "nullish"],
      [20, "nullish"],
      [24, "nullish"],
      [41, "nullish"],
      [54, "nullish"],
      [60, "nullish"],
    ]);
    expect(
      cases
        .find((item) => item.name === "scope edges keep each ReactNode binding")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [13, "nullish"],
      [34, "unanalyzable"],
      [53, "nullish"],
      [79, "nullish"],
      [95, "nullish"],
      [99, "nullish"],
      [103, "nullish"],
      [109, "unanalyzable"],
      [121, "nullish"],
      [134, "nullish"],
      [138, "nullish"],
      [163, "nullish"],
      [167, "nullish"],
      [179, "nullish"],
      [188, "nullish"],
      [192, "nullish"],
      [197, "nullish"],
      [201, "nullish"],
    ]);
    expect(
      cases
        .find((item) => item.name === "numeric literal keys stay on that property")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [6, "nullish"],
      [10, "nullish"],
      [19, "nullish"],
      [23, "nullish"],
      [28, "unanalyzable"],
      [32, "nullish"],
    ]);
    expect(
      cases
        .find((item) => item.name === "erased require and a later assignment")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "wrapper"],
      [10, "wrapper"],
      [16, "wrapper"],
      [31, "unanalyzable"],
      [37, "unanalyzable"],
      [43, "wrapper"],
      [55, "unanalyzable"],
      [61, "wrapper"],
      [76, "unanalyzable"],
      [82, "wrapper"],
      [88, "unanalyzable"],
      [94, "unanalyzable"],
    ]);
    expect(
      cases.find((item) => item.name === "reassigned alias chain stays bounded").findings,
    ).toEqual([]);
    expect(
      cases
        .find((item) => item.name === "named import member chain")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "wrapper"],
      [10, "wrapper"],
    ]);
    expect(
      cases
        .find((item) => item.name === "default member alias keeps its prefix")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [7, "wrapper"],
      [11, "wrapper"],
      [20, "wrapper"],
    ]);
    expect(
      cases
        .find((item) => item.name === "nested destructure keeps the member path")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [8, "wrapper"],
      [15, "unanalyzable"],
      [22, "unanalyzable"],
      [27, "unanalyzable"],
      [34, "unanalyzable"],
      [39, "wrapper"],
      [44, "unanalyzable"],
    ]);
    expect(
      cases
        .find((item) => item.name === "unresolved qualified getter terminal")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [5, "unanalyzable"],
      [14, "wrapper"],
      [18, "unanalyzable"],
      [24, "wrapper"],
      [31, "wrapper"],
      [37, "wrapper"],
    ]);
    expect(
      cases
        .find((item) => item.name === "computed member of a namespace import")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [6, "unanalyzable"],
      [10, "wrapper"],
      [14, "wrapper"],
    ]);
    expect(
      cases
        .find((item) => item.name === "unresolved wrapper is not inside")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [6, "unanalyzable"],
      [6, "unanalyzable"],
      [12, "unanalyzable"],
      [23, "unanalyzable"],
      [23, "unanalyzable"],
      [27, "unanalyzable"],
      [27, "wrapper"],
      [31, "unanalyzable"],
      [31, "wrapper"],
      [35, "unanalyzable"],
      [35, "unanalyzable"],
      [41, "unanalyzable"],
      [41, "unanalyzable"],
      [49, "wrapper"],
    ]);
    expect(
      cases
        .find((item) => item.name === "ignored unresolved wrapper")
        .findings.map((finding) => [finding.line, finding.messageId]),
    ).toEqual([
      [27, "wrapper"],
      [31, "wrapper"],
      [49, "wrapper"],
    ]);

    const oxlint = {
      "react-use": oxlintOn(".oxlintrc-react-use.json", [
        "react-use-cjs.tsx",
        "react-use-wrappers.tsx",
        "react-use-unresolved.tsx",
        "react-use-bindings.tsx",
        "react-use-alias-chain.tsx",
        "promise-type-query.tsx",
        "react-use-type-require.tsx",
        "react-use-value-require.tsx",
        "react-use-inner-type-require.tsx",
        "react-use-binding-edges.tsx",
      ]),
      nullish: oxlintOn(".oxlintrc-nullish.json", [
        "react-nullish.tsx",
        "react-nullish-unresolved.tsx",
        "react-nullish-members.tsx",
        "react-nullish-local-require.tsx",
        "react-nullish-annotated.tsx",
        "react-nullish-alias-chain.tsx",
        "react-nullish-pattern-shadows.tsx",
        "react-nullish-assertions.ts",
        "react-nullish-named-default.tsx",
        "react-nullish-let-require.tsx",
        "react-nullish-computed-props.tsx",
        "react-nullish-merged.tsx",
        "react-nullish-loop-class.tsx",
        "react-nullish-self-alias.tsx",
        "react-nullish-nested-heritage.tsx",
        "react-nullish-binding-shadows.tsx",
        "react-nullish-scope-edges.tsx",
        "react-nullish-numeric-keys.tsx",
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
        "src/escaped-script-import.tsx",
        "app/script-spread.tsx",
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
        "nullable-relay-getter.ts",
        "nullable-member-alias.ts",
        "nullable-forwarded-member.ts",
        "nullable-unresolved-name.ts",
        "nullable-computed-member.ts",
        "nullable-unresolved-wrapper.ts",
        "nullable-binding-shapes.ts",
        "nullable-assigned-require.ts",
      ]),
      "nullable-qualified": oxlintOn(".oxlintrc-nullable-qualified.json", [
        "nullable-qualified-member.ts",
        "nullable-named-chain.ts",
        "nullable-nested-destructure.ts",
      ]),
      "nullable-default": oxlintOn(".oxlintrc-nullable-default.json", [
        "nullable-default-alias.ts",
      ]),
      "nullable-ignore": oxlintOn(".oxlintrc-nullable-ignore.json", [
        "nullable-relay-getter.ts",
        "nullable-unresolved-name.ts",
        "nullable-unresolved-wrapper.ts",
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
    assert.deepEqual(
      oxlint["nullable-ignore"].findings.map((item) => [
        item.file,
        item.findings.map((finding) => [finding.line, finding.message]),
      ]),
      [
        ["nullable-relay-getter.ts", []],
        ["nullable-unresolved-name.ts", []],
        [
          "nullable-unresolved-wrapper.ts",
          [
            [
              27,
              "Wrap nullable entity fetch 'getWidget' in wrapNullable so missing entities map to null consistently.",
            ],
            [
              31,
              "Wrap nullable entity fetch 'getWidget' in wrapNullable so missing entities map to null consistently.",
            ],
            [
              49,
              "Wrap nullable entity fetch 'getWidget' in wrapNullable so missing entities map to null consistently.",
            ],
          ],
        ],
      ],
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
      oxlint["nullable-outer"].status,
      0,
      JSON.stringify(oxlint["nullable-outer"].findings),
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
