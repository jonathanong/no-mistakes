import assert from "node:assert/strict";
import { describe, expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { fixture, plugin } from "./helpers.mjs";

describe("plugin exports", () => {
  it("exposes rules and flat configs", () => {
    assert.equal(plugin.meta.name, "eslint-plugin-no-mistakes");
    assert.ok(plugin.rules["playwright-literals"]);
    assert.equal(plugin.configs.recommended.rules["no-mistakes/playwright-literals"], "error");
    assert.deepEqual(plugin.configs.strict.rules["no-mistakes/playwright-consistent-attribute"], [
      "error",
      { canonicalAttribute: "data-pw" },
    ]);
    assert.equal(plugin.configs.strict.rules["no-mistakes/test-no-delayed-rejects"], "error");
    assert.equal(
      plugin.configs.recommended.rules["no-mistakes/test-no-delayed-rejects"],
      undefined,
    );
  });
});

function lintCase(name, code, rule, option) {
  return {
    name,
    rule,
    ...(option !== undefined ? { options: option } : {}),
    findings: pluginFindings(code, rule, option),
  };
}

it("snapshots rules diagnostics", async () => {
  const staticTemplates = { allowStaticTemplates: true };
  const literalOnly = {
    selectorAttributes: ["data-qa"],
    allowDefaultedProps: false,
  };
  const canonicalQa = {
    selectorAttributes: ["data-qa"],
    canonicalAttribute: "data-qa",
  };
  const cases = [
    lintCase(
      "accepts literals, expression literals, empty static templates, static templates, and defaulted props",
      fixture("literals-valid.jsx"),
      "playwright-literals",
      staticTemplates,
    ),
    lintCase(
      "reports missing, dynamic, non-defaulted, and forbidden template values",
      fixture("literals-invalid.jsx"),
      "playwright-literals",
    ),
    lintCase(
      "rejects templates without static text and allows defaulted identifiers outside props",
      `
      function A(testId = "save") {
        helper();
        page.getByTestId(testId);
        return <button data-pw={\`\${id}\`} />;
      }
    `,
      "playwright-literals",
      staticTemplates,
    ),
    lintCase(
      "can be configured for literal-only mode and custom attributes",
      `
      const A = ({ testId = "save" }) => <button data-qa={testId} />;
    `,
      "playwright-literals",
      literalOnly,
    ),
    lintCase(
      "does not treat shadowed locals as defaulted props",
      `
      function A({ testId = "save" }) {
        {
          const testId = id;
          return <button data-pw={testId} />;
        }
      }
    `,
      "playwright-literals",
    ),
    lintCase(
      "accepts defaulted props through nested scopes only when unshadowed",
      `
      function A({ testId = "save" }) {
        {
          return <button data-pw={testId} />;
        }
      }
    `,
      "playwright-literals",
    ),
    lintCase(
      "accepts const destructured literal defaults",
      `function A(props) { const { "data-pw": dataPw = "save", nested: { nestedPw = "open" } = {} } = props; const [, arrayPw = "array"] = props.items; page.getByTestId(dataPw); if (props.ready) { const { readyPw = "ready" } = props; return <button data-pw={readyPw} />; } return <><button data-pw={dataPw} /><button data-pw={nestedPw} /><button data-pw={arrayPw} /></>; }`,
      "playwright-literals",
    ),
    lintCase(
      "rejects unsafe const destructured defaults",
      `function A(props) { function B() { const { "data-pw": inner = "inner" } = props; } const { "data-pw": missing } = props; const { "data-testid": dynamic = id } = props; let { "data-qa": mutable = "open" } = props; return <><button data-pw={inner} /><button data-pw={missing} /><button data-pw={dynamic} /><button data-pw={mutable} /></>; }`,
      "playwright-literals",
    ),
    lintCase(
      "does not treat rest bindings as defaulted props",
      `function A(props) { const { ...rest } = props; return <button data-pw={rest} />; }`,
      "playwright-literals",
    ),
    lintCase(
      "rejects missing getByTestId arguments and identifiers outside functions",
      "page.getByTestId(); page.getByTestId(testId);",
      "playwright-literals",
    ),
    lintCase(
      "requires literal defaults for prop passthrough",
      fixture("defaults.jsx"),
      "playwright-defaults",
    ),
    lintCase(
      "rejects shadowed defaulted props",
      `
      function A({ testId = "save" }) {
        {
          const testId = id;
          return <button data-pw={testId} />;
        }
      }
    `,
      "playwright-defaults",
    ),
    lintCase(
      "accepts const destructured literal defaults",
      `function A(props) { const { "data-pw": dataPw = "save" } = props; return <button data-pw={dataPw} />; }`,
      "playwright-defaults",
    ),
    lintCase(
      "rejects shadowed and unsafe const destructured defaults",
      `function A(props) { const { "data-pw": dataPw = "save" } = props; const { "data-testid": dynamic = id } = props; let { "data-qa": mutable = "open" } = props; { const dataPw = id; return <><button data-pw={dataPw} /><button data-pw={dynamic} /><button data-pw={mutable} /></>; } }`,
      "playwright-defaults",
    ),
    lintCase(
      "rejects const defaults from non-props, before declaration, and mismatched parameters",
      `function A(props, cfg) { const { "data-pw": fromCfg = "cfg" } = cfg; const before = <button data-pw={late} />; const { "data-pw": late = "late" } = props; return <><button data-pw={fromCfg} />{before}</>; } function B(testId, props) { { const { "data-pw": testId = "save" } = props; } return <button data-pw={testId} />; }`,
      "playwright-defaults",
    ),
    lintCase(
      "reports duplicate exact literals within a file: unique.jsx",
      fixture("unique.jsx"),
      "playwright-unique",
    ),
    lintCase(
      "reports duplicate exact literals within a file: dynamic id",
      "<button data-pw={id} />;",
      "playwright-unique",
    ),
    lintCase(
      "requires the configured canonical attribute: data-testid",
      "<button data-testid='save' />;",
      "playwright-consistent-attribute",
    ),
    lintCase(
      "requires the configured canonical attribute: data-pw",
      "<button data-pw='save' />;",
      "playwright-consistent-attribute",
    ),
    lintCase(
      "requires the configured canonical attribute: data-qa",
      "<button data-qa='save' />;",
      "playwright-consistent-attribute",
      canonicalQa,
    ),
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot("snapshots/rules.snap");
});
