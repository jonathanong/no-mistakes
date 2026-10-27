import assert from "node:assert/strict";
import { expect, it } from "vitest";
import {
  pluginFindings,
  snapshotCases,
  snapshotDocument,
  snapshotSuite,
} from "./diagnostic-snapshot.mjs";
import { plugin } from "./helpers.mjs";

const ruleName = "playwright-require-exported-component-attribute";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  parserOptions: { ecmaFeatures: { jsx: true } },
};

const suite = {
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
    },
    {
      code: "export const Button = () => <button />;",
    },
    {
      code: "function Button() { return <button />; } export { Button };",
    },
    {
      code: "export default function Button() { return <button />; }",
    },
    {
      code: "function Button() { return <button />; } export { Button as default };",
    },
    {
      code: "const Button = memo(forwardRef(() => <button />)); export { Button };",
    },
    {
      code: "export function Button() { if (ready) return <button />; return <button data-pw='save' />; }",
    },
    {
      code: "export function Button() { return ready ? <button /> : <button data-pw='save' />; }",
    },
    {
      code: "export function Button() { function helper() { return <button data-pw='nested' />; } return <button />; }",
    },
    {
      code: "export function Button() { return <button aria-label='Save' />; }",
      options: [{ attributes: ["data-pw"] }],
    },
    {
      code: "export default function () { return <button />; }",
      options: [{ checkAnonymousDefault: true }],
    },
    {
      code: "export function Button() { return <button {...props} />; }",
    },
    {
      code: "export function SaveButton() { return <button />; } export function CancelLink() { return <a data-pw='cancel' />; }",
      options: [{ components: ["/Button$/"] }],
    },
  ],
};

it("snapshots playwright-require-exported-component-attribute diagnostics", async () => {
  await expect(
    snapshotDocument({
      messages: snapshotCases([
        {
          findings: pluginFindings(
            "export function Button() { return ready ? <button /> : <a />; }",
            ruleName,
          ),
          name: "reports one missing branch per JSX return branch",
        },
        {
          findings: pluginFindings(
            "export function Button() { return ready ? <button data-pw='save' /> : null; }",
            ruleName,
          ),
          name: "ignores null branches",
        },
      ]),
      ruleTester: snapshotSuite(ruleName, languageOptions, suite),
    }),
  ).toMatchFileSnapshot("snapshots/require-exported-component-attribute.snap");
});

it("is exported by the plugin", () => {
  assert.ok(plugin.rules["playwright-require-exported-component-attribute"]);
});
