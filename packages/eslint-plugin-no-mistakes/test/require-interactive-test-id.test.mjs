import { expect, it } from "vitest";
import {
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const ruleName = "playwright-require-interactive-test-id";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
  valid: [
    { code: "<a />;" },
    { code: "<Foo.bar data-testid='id' />;" },
    { code: "<a onClick={fn} href='/x' data-testid='id' />;" },
    { code: "<div onClick={fn} data-testid='id' />;" },
    { code: "<div role='button' data-testid='id' />;" },
    { code: "<div role='presentation' />;" },
    { code: "<div {...props} />;" },
    { code: "<Comp.Button />;" },
    { code: "<Button />;" },
    {
      code: "<Button data-pw='save' />;",
      options: [{ interactiveComponents: ["Button"] }],
    },
    {
      code: "<Ui.Button data-pw='save' />;",
      options: [{ interactiveComponents: ["Ui.Button"] }],
    },
    { code: "<a>link</a>;" },
    { code: "<div className='x' />;" },
    { code: "<div role='button' data-pw={id} />;" },
    {
      code: `<>
      <button data-pw="save" />
      <input data-testid="input" />
      <a href="/x" data-pw="link" />
      <div onClick={fn} data-pw="click" />
      <div role="button" data-pw="btn" />
    </>`,
    },
    {
      code: `<button data-qa="save" />`,
      options: [{ selectorAttributes: ["data-qa"] }],
    },
  ],
  invalid: [
    {
      // Exercise slots through the direct rule entrypoint as well as the plugin loader.
      code: fixture("playwright-require-interactive-test-id/slots-missing.tsx"),
      options: [{ interactiveComponents: ["Button"] }],
    },
    {
      code: fixture("playwright-require-interactive-test-id/slots-controls.tsx"),
      options: [{ interactiveComponents: ["Button"] }],
    },
    {
      code: fixture("playwright-require-interactive-test-id/branches.jsx"),
      options: [{ interactiveComponents: ["/(/"] }],
    },
    {
      code: fixture("playwright-require-interactive-test-id/branches.jsx"),
      options: [{ interactiveComponents: ["Button", "Card"] }],
    },
    {
      code: "<div role='button' />;",
    },
    {
      code: "<button />;",
    },
    {
      code: "<div className='x' role='button' />;",
    },
    {
      code: "<Comp.Button onClick={fn} />;",
    },
    {
      code: "<div onClick={fn} role='presentation' />;",
    },
    {
      code: "<a href='/x' />;",
    },
    {
      code: "<Foo.Bar onClick={handler} />;",
    },
    {
      code: "<><button /><input /><select /><textarea /></>;",
    },
    {
      code: "<><a href='/x' /><a>link</a></>;",
    },
    {
      code: "<div onClick={() => {}} />;",
    },
    {
      code: "<Link href='/login'>Create your free page</Link>;",
      options: [{ interactiveComponents: ["Link"] }],
    },
    {
      code: "<Button type='submit'>Submit</Button>;",
      options: [{ interactiveComponents: ["Button"] }],
    },
    {
      code: "<SelectItem value='incorrect_facts'>The facts cited are incorrect</SelectItem>;",
      options: [{ interactiveComponents: ["SelectItem"] }],
    },
    {
      code: "<Ui.Button>Submit</Ui.Button>;",
      options: [{ interactiveComponents: ["Ui.Button"] }],
    },
    {
      code: "<Menu.Item>Open</Menu.Item>;",
      options: [{ interactiveComponents: ["/\\.Item$/"] }],
    },
    {
      code: "<><Button /><Button /></>;",
      options: [{ interactiveComponents: ["/^Button$/g"] }],
    },
    {
      code: [
        "<>",
        '  <div role="button" />',
        '  <div role="checkbox" />',
        '  <div role="link" />',
        '  <div role="menuitem" />',
        '  <div role="option" />',
        '  <div role="radio" />',
        '  <div role="switch" />',
        '  <div role="tab" />',
        '  <div role="textbox" />',
        "</>",
      ].join("\n"),
    },
    {
      code: '<>\n<button data-qa="save" />\n<button data-pw="save" />\n</>',
      options: [{ selectorAttributes: ["data-qa"] }],
    },
  ],
};

it("snapshots playwright-require-interactive-test-id diagnostics", async () => {
  await expect(
    snapshotDocument({
      messages: snapshotCases([
        {
          findings: pluginFindings("<a href='/x' />;", ruleName),
          name: "reports anchor href without selectors",
        },
        {
          findings: pluginFindings("<div onClick={fn} />;", ruleName),
          name: "reports onClick controls without selectors",
        },
        {
          findings: pluginFindings("<div role='presentation' />;", ruleName),
          name: "ignores non-interactive role without onClick",
        },
        {
          findings: pluginFindings("<button />;", ruleName),
          name: "covers intrinsic controls",
        },
        {
          findings: pluginFindings("<span title='copy' />;", ruleName),
          name: "covers non-role attributes",
        },
        {
          findings: pluginFindings("<Button />;", ruleName, { interactiveComponents: ["Button"] }),
          name: "reports configured Button matcher",
          options: { interactiveComponents: ["Button"] },
        },
        {
          findings: pluginFindings("<Menu.Item />;", ruleName, {
            interactiveComponents: ["/\\.Item$/"],
          }),
          name: "reports configured Menu.Item matcher",
          options: { interactiveComponents: ["/\\.Item$/"] },
        },
        {
          findings: pluginFindings("<Button />;", ruleName, { interactiveComponents: [""] }),
          name: "ignores empty configured component matchers",
          options: { interactiveComponents: [""] },
        },
        {
          findings: pluginFindings("<svg:path />;", ruleName, {
            interactiveComponents: ["svg:path"],
          }),
          name: "ignores unsupported JSX names",
          options: { interactiveComponents: ["svg:path"] },
        },
        {
          findings: pluginFindings("<Button />;", ruleName, { interactiveComponents: ["/(/"] }),
          name: "ignores invalid component matchers",
          options: { interactiveComponents: ["/(/"] },
        },
      ]),
      ruleTester: snapshotSuite(ruleName, languageOptions, suite),
    }),
  ).toMatchFileSnapshot("snapshots/require-interactive-test-id.snap");
});
