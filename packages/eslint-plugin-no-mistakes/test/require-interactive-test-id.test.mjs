import { RuleTester } from "eslint";
import assert from "node:assert/strict";
import { afterAll, describe, it } from "vitest";
import rule from "../src/rules/playwright-require-interactive-test-id.js";
import { messages } from "./helpers.mjs";

RuleTester.describe = describe;
RuleTester.it = it;
RuleTester.itOnly = it.only;
RuleTester.afterAll = afterAll;

const tester = new RuleTester({
  languageOptions: {
    ecmaVersion: 2024,
    sourceType: "module",
    parserOptions: { ecmaFeatures: { jsx: true } },
  },
});

tester.run("playwright-require-interactive-test-id", rule, {
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
      code: "<div role='button' />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<button />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<div className='x' role='button' />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<Comp.Button onClick={fn} />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<div onClick={fn} role='presentation' />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<a href='/x' />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<Foo.Bar onClick={handler} />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<><button /><input /><select /><textarea /></>;",
      errors: [
        { messageId: "missing" },
        { messageId: "missing" },
        { messageId: "missing" },
        { messageId: "missing" },
      ],
    },
    {
      code: "<><a href='/x' /><a>link</a></>;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<div onClick={() => {}} />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<Link href='/login'>Create your free page</Link>;",
      options: [{ interactiveComponents: ["Link"] }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<Button type='submit'>Submit</Button>;",
      options: [{ interactiveComponents: ["Button"] }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<SelectItem value='incorrect_facts'>The facts cited are incorrect</SelectItem>;",
      options: [{ interactiveComponents: ["SelectItem"] }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<Ui.Button>Submit</Ui.Button>;",
      options: [{ interactiveComponents: ["Ui.Button"] }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<Menu.Item>Open</Menu.Item>;",
      options: [{ interactiveComponents: ["/\\.Item$/"] }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "<><Button /><Button /></>;",
      options: [{ interactiveComponents: ["/^Button$/g"] }],
      errors: [{ messageId: "missing" }, { messageId: "missing" }],
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
      errors: [
        { messageId: "missing", line: 2, column: 4 },
        { messageId: "missing", line: 3, column: 4 },
        { messageId: "missing", line: 4, column: 4 },
        { messageId: "missing", line: 5, column: 4 },
        { messageId: "missing", line: 6, column: 4 },
        { messageId: "missing", line: 7, column: 4 },
        { messageId: "missing", line: 8, column: 4 },
        { messageId: "missing", line: 9, column: 4 },
        { messageId: "missing", line: 10, column: 4 },
      ],
    },
    {
      code: '<>\n<button data-qa="save" />\n<button data-pw="save" />\n</>',
      options: [{ selectorAttributes: ["data-qa"] }],
      errors: [{ messageId: "missing", line: 3, column: 2 }],
    },
  ],
});

describe("messages coverage", () => {
  it("reports anchor href and onClick controls without selectors", () => {
    assert.deepEqual(messages("<a href='/x' />;", "playwright-require-interactive-test-id"), [
      "missing",
    ]);
    assert.deepEqual(messages("<div onClick={fn} />;", "playwright-require-interactive-test-id"), [
      "missing",
    ]);
  });

  it("ignores non-interactive role without onClick", () => {
    assert.deepEqual(
      messages("<div role='presentation' />;", "playwright-require-interactive-test-id"),
      [],
    );
  });

  it("covers intrinsic controls and non-role attributes", () => {
    assert.deepEqual(messages("<button />;", "playwright-require-interactive-test-id"), [
      "missing",
    ]);
    assert.deepEqual(
      messages("<span title='copy' />;", "playwright-require-interactive-test-id"),
      [],
    );
  });

  it("reports configured component matchers", () => {
    assert.deepEqual(
      messages("<Button />;", "playwright-require-interactive-test-id", {
        interactiveComponents: ["Button"],
      }),
      ["missing"],
    );
    assert.deepEqual(
      messages("<Menu.Item />;", "playwright-require-interactive-test-id", {
        interactiveComponents: ["/\\.Item$/"],
      }),
      ["missing"],
    );
  });

  it("ignores empty configured component matchers and unsupported JSX names", () => {
    assert.deepEqual(
      messages("<Button />;", "playwright-require-interactive-test-id", {
        interactiveComponents: [""],
      }),
      [],
    );
    assert.deepEqual(
      messages("<svg:path />;", "playwright-require-interactive-test-id", {
        interactiveComponents: ["svg:path"],
      }),
      [],
    );
    assert.deepEqual(
      messages("<Button />;", "playwright-require-interactive-test-id", {
        interactiveComponents: ["/(/"],
      }),
      [],
    );
  });
});
