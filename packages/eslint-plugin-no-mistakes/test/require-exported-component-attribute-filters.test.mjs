import { expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const RULE = "playwright-require-exported-component-attribute";
const DIR = "playwright-require-exported-component-attribute";

function filterCase(name, file, option) {
  return {
    name,
    file,
    rule: RULE,
    ...(option !== undefined ? { options: option } : {}),
    findings: pluginFindings(fixture(`${DIR}/${file}`), RULE, option, file),
  };
}

it("snapshots require-exported-component-attribute-filters diagnostics", async () => {
  const cases = [
    filterCase("does not check ignored exported components", "ignore-button.jsx", {
      ignoreComponents: ["Button"],
    }),
    filterCase(
      "does not check lowercase hooks under the default name pattern",
      "lowercase-hook.jsx",
    ),
    filterCase("checks only listed components when components is set: Card", "unlisted-card.jsx", {
      components: ["Card"],
    }),
    filterCase(
      "checks only listed components when components is set: Button",
      "unlisted-card.jsx",
      { components: ["Button"] },
    ),
    // Button already has data-pw, so a default PascalCase run is silent.
    // The configured run can report missing only if widget is selected.
    filterCase(
      "applies componentNamePattern instead of the default PascalCase filter: widget",
      "pattern-widget.jsx",
      { componentNamePattern: "^widget$" },
    ),
    filterCase(
      "applies componentNamePattern instead of the default PascalCase filter: default",
      "pattern-widget.jsx",
    ),
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/require-exported-component-attribute-filters.snap",
  );
});
