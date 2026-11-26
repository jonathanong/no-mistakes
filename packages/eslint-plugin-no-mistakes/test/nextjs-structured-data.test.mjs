import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, lint, plugin } from "./helpers.mjs";

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/nextjs-structured-data/fixture", name),
    "utf8",
  );
}

const ruleName = "no-mistakes/nextjs-no-manual-script-tags";
const languageOptions = { parser: tsParser, sourceType: "module", globals: {} };

function lintCase(code, filename, option) {
  return { code, filename, ...(option === undefined ? {} : { options: [option] }) };
}

describe("nextjs structured data opt-in", () => {
  it("snapshots default allowance, opt-in diagnostics, and unchanged executable scripts", async () => {
    const jsonld = fixture("app/page.tsx");
    const inlineBoot = readFileSync(
      resolve(
        __dirname,
        "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/next-script-inline-boot.tsx",
      ),
      "utf8",
    );
    const component = { structuredDataComponent: "StructuredData" };
    const custom = { structuredDataMessage: "Render structured data with the approved component." };
    await expect(
      snapshotDocument(
        snapshotSuite("nextjs-no-manual-script-tags", languageOptions, {
          valid: [
            lintCase(jsonld, "app/page.tsx"),
            lintCase(jsonld, "app/page.tsx", {}),
            lintCase(jsonld, "components/metadata.tsx", component),
            lintCase(fixture("component.tsx"), "app/page.tsx", component),
            lintCase(inlineBoot, "app/layout.tsx", {
              ...component,
              allowInlineScriptIds: ["theme-init"],
            }),
          ],
          invalid: [
            lintCase(jsonld, "app/page.tsx", component),
            lintCase(jsonld, "app/page.tsx", custom),
            lintCase(jsonld, "app/page.tsx", { ...component, ...custom }),
            lintCase(jsonld, "app/page.tsx", {
              ...component,
              allowInlineScriptIds: ["page-schema", "secondary-schema"],
            }),
            lintCase(jsonld, "app/page.tsx", {
              ...component,
              allowInlineScriptIdPatterns: ["-schema$"],
            }),
            lintCase(fixture("next-import.tsx"), "components/metadata.tsx", component),
            lintCase(fixture("other-types.tsx"), "app/page.tsx", { ...component, ...custom }),
            lintCase(inlineBoot, "app/layout.tsx", { ...component, ...custom }),
          ],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/nextjs-structured-data.snap");
  });

  it("uses flat-config files and ignores to scope the opt-in and exclude its wrapper", () => {
    const linter = new Linter({ configType: "flat" });
    const config = [
      { files: ["**/*.tsx"], languageOptions },
      {
        files: ["app/**/*.tsx"],
        ignores: ["app/structured-data.tsx"],
        plugins: { "no-mistakes": plugin },
        rules: { [ruleName]: ["error", { structuredDataComponent: "StructuredData" }] },
      },
    ];
    expect(
      linter.verify(fixture("app/page.tsx"), config, "app/page.tsx").map((item) => item.messageId),
    ).toEqual(["structuredData", "structuredData"]);
    expect(linter.verify(fixture("app/page.tsx"), config, "pages/page.tsx")).toEqual([]);
    expect(
      linter.verify(fixture("structured-data.tsx"), config, "app/structured-data.tsx"),
    ).toEqual([]);
  });

  it.each([
    { structuredDataComponent: "" },
    { structuredDataComponent: false },
    { structuredDataMessage: "" },
    { structuredDataMessage: 42 },
    { structuredDataComponent: "StructuredData", unknown: true },
  ])("rejects invalid structured-data options %j", (options) => {
    expect(() =>
      lint(fixture("app/page.tsx"), { [ruleName]: ["error", options] }, "app/page.tsx"),
    ).toThrow();
  });
});
