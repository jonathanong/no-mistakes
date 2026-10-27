import { expect, it } from "vitest";
import { pluginFindings, snapshotCases, snapshotDocument } from "./diagnostic-snapshot.mjs";

const RULE = "playwright-require-exported-component-attribute";

function extraCase(name, code, option, filename = "fixture.jsx") {
  return {
    name,
    file: filename,
    rule: RULE,
    ...(option !== undefined ? { options: option } : {}),
    findings: pluginFindings(code, RULE, option, filename),
  };
}

it("snapshots require-exported-component-attribute-extra diagnostics", async () => {
  const cases = [
    extraCase(
      "ignores exported variables without function initializers: let",
      "export let Button;",
    ),
    extraCase(
      "ignores exported variables without function initializers: memo identifier",
      "export const Button = memo(Component);",
    ),
    extraCase(
      "ignores exported variables without function initializers: destructure",
      "export const { Button } = things;",
    ),
    extraCase(
      "reports a component wrapped in a static computed memo call",
      "export const Button = React['memo'](() => <button />);",
    ),
    extraCase(
      "unwraps syntax wrappers around configured wrapper arguments",
      `
      export const Button = memo((() => <button data-pw="save" />) as React.FC);
    `,
      undefined,
      "fixture.tsx",
    ),
    extraCase(
      "checks default exported wrapped named functions",
      `
      export default memo(function Button() {
        return <button />;
      });
    `,
    ),
    extraCase(
      "checks anonymous default exported expressions when enabled",
      "export default (() => <button />);",
      { checkAnonymousDefault: true },
    ),
    extraCase(
      "deduplicates components exported inline and by specifier",
      `
      export function Button() {
        return <button />;
      }
      export { Button as ButtonAlias };
    `,
    ),
    extraCase(
      "ignores returns inside nested classes",
      `
      export function Button() {
        class Inner {
          render() {
            return <span />;
          }
        }
        return <button data-pw="save" />;
      }
    `,
    ),
    extraCase(
      "checks logical expression returns",
      "export function Button() { return ready && <button />; }",
    ),
    extraCase(
      "checks sequence expression returns",
      "export function Button() { return (track(), <button />); }",
    ),
    extraCase(
      "checks array expression returns",
      "export function Button() { return [<button />]; }",
    ),
    extraCase(
      "unwraps TypeScript syntax wrappers around returned JSX",
      `
      export function Button() {
        return <button /> as JSX.Element;
      }
    `,
      undefined,
      "fixture.tsx",
    ),
  ];
  await expect(snapshotDocument(snapshotCases(cases))).toMatchFileSnapshot(
    "snapshots/require-exported-component-attribute-extra.snap",
  );
});
