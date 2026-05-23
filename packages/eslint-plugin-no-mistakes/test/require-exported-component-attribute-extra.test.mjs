import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";

const RULE = "playwright-require-exported-component-attribute";

describe("require-exported-component-attribute edge cases", () => {
  it("ignores exported variables without function initializers", () => {
    assert.deepEqual(messages("export let Button;", RULE), []);
    assert.deepEqual(messages("export const Button = memo(Component);", RULE), []);
    assert.deepEqual(messages("export const Button = React['memo'](() => <button />);", RULE), []);
    assert.deepEqual(messages("export const { Button } = things;", RULE), []);
  });

  it("unwraps syntax wrappers around configured wrapper arguments", () => {
    const code = `
      export const Button = memo((() => <button data-pw="save" />) as React.FC);
    `;
    assert.deepEqual(messages(code, RULE, undefined, "fixture.tsx"), []);
  });

  it("checks default exported wrapped named functions", () => {
    const code = `
      export default memo(function Button() {
        return <button />;
      });
    `;
    assert.deepEqual(messages(code, RULE), ["missing"]);
  });

  it("checks anonymous default exported expressions when enabled", () => {
    const code = "export default (() => <button />);";
    assert.deepEqual(messages(code, RULE, { checkAnonymousDefault: true }), ["missing"]);
  });

  it("deduplicates components exported inline and by specifier", () => {
    const code = `
      export function Button() {
        return <button />;
      }
      export { Button as ButtonAlias };
    `;
    assert.deepEqual(messages(code, RULE), ["missing"]);
  });

  it("ignores returns inside nested classes", () => {
    const code = `
      export function Button() {
        class Inner {
          render() {
            return <span />;
          }
        }
        return <button data-pw="save" />;
      }
    `;
    assert.deepEqual(messages(code, RULE), []);
  });

  it("checks logical expression returns", () => {
    assert.deepEqual(messages("export function Button() { return ready && <button />; }", RULE), [
      "missing",
    ]);
  });

  it("checks sequence expression returns", () => {
    assert.deepEqual(messages("export function Button() { return (track(), <button />); }", RULE), [
      "missing",
    ]);
  });

  it("checks array expression returns", () => {
    assert.deepEqual(messages("export function Button() { return [<button />]; }", RULE), [
      "missing",
    ]);
  });

  it("unwraps TypeScript syntax wrappers around returned JSX", () => {
    const code = `
      export function Button() {
        return <button /> as JSX.Element;
      }
    `;
    assert.deepEqual(messages(code, RULE, undefined, "fixture.tsx"), ["missing"]);
  });
});
