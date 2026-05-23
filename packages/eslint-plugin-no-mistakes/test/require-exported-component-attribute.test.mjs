import { RuleTester } from "eslint";
import assert from "node:assert/strict";
import { afterAll, describe, it } from "vitest";
import rule from "../src/rules/playwright-require-exported-component-attribute.js";
import { messages, plugin } from "./helpers.mjs";

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

tester.run("playwright-require-exported-component-attribute", rule, {
  valid: [
    { code: "export function Button() { return <button data-pw='save' />; }" },
    { code: "export const Button = () => <button><span data-pw='save' /></button>;" },
    { code: "function Button() { return <button data-pw='save' />; } export { Button };" },
    { code: "const Button = memo(() => <button data-pw='save' />); export { Button };" },
    {
      code: "const Button = React.forwardRef(() => <button data-pw='save' />); export default Button;",
    },
    {
      code: "export default function Button() { return ready ? <button data-pw='save' /> : null; }",
    },
    { code: "function Button() { return <button />; }" },
    { code: "export function useButton() { return <button />; }" },
    { code: "export { Button } from './button';" },
    {
      code: "export function Button() { return <button aria-label='Save' />; }",
      options: [{ attributes: ["data-pw", "aria-label"] }],
    },
    {
      code: "export function Button() { return <button />; }",
      options: [{ ignoreComponents: ["Button"] }],
    },
    {
      code: "export function Button() { return <button />; }",
      options: [{ components: ["Card"] }],
    },
    {
      code: "export function widget() { return <button data-pw='save' />; }",
      options: [{ componentNamePattern: "^widget$" }],
    },
    {
      code: "export function Button() { return <button {...props} />; }",
      options: [{ allowSpreadAttributes: true }],
    },
    {
      code: "export function Button() { return <><button data-pw='save' /><button /></>; }",
    },
    {
      code: "export default function Button() { return <button />; }",
      options: [{ exportTypes: ["named"] }],
    },
    {
      code: "function Button() { return <button />; } export { Button as default };",
      options: [{ exportTypes: ["named"] }],
    },
    {
      code: "export default function () { return <button data-pw='save' />; }",
      options: [{ checkAnonymousDefault: true }],
    },
    {
      code: "export const Button = customWrapper(() => <button data-pw='save' />);",
      options: [{ wrappers: ["customWrapper"] }],
    },
  ],
  invalid: [
    {
      code: "export function Button() { return <button />; }",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export const Button = () => <button />;",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "function Button() { return <button />; } export { Button };",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export default function Button() { return <button />; }",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "function Button() { return <button />; } export { Button as default };",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "const Button = memo(forwardRef(() => <button />)); export { Button };",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export function Button() { if (ready) return <button />; return <button data-pw='save' />; }",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export function Button() { return ready ? <button /> : <button data-pw='save' />; }",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export function Button() { function helper() { return <button data-pw='nested' />; } return <button />; }",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export function Button() { return <button aria-label='Save' />; }",
      options: [{ attributes: ["data-pw"] }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export default function () { return <button />; }",
      options: [{ checkAnonymousDefault: true }],
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export function Button() { return <button {...props} />; }",
      errors: [{ messageId: "missing" }],
    },
    {
      code: "export function SaveButton() { return <button />; } export function CancelLink() { return <a data-pw='cancel' />; }",
      options: [{ components: ["/Button$/"] }],
      errors: [{ messageId: "missing" }],
    },
  ],
});

describe("require-exported-component-attribute", () => {
  it("is exported by the plugin", () => {
    assert.ok(plugin.rules["playwright-require-exported-component-attribute"]);
  });

  it("reports one missing branch per JSX return branch", () => {
    assert.deepEqual(
      messages(
        "export function Button() { return ready ? <button /> : <a />; }",
        "playwright-require-exported-component-attribute",
      ),
      ["missing", "missing"],
    );
  });

  it("ignores null branches", () => {
    assert.deepEqual(
      messages(
        "export function Button() { return ready ? <button data-pw='save' /> : null; }",
        "playwright-require-exported-component-attribute",
      ),
      [],
    );
  });
});
