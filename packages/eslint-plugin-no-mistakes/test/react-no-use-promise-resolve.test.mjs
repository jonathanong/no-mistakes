import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";

const rule = "react-no-use-promise-resolve";
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function useCase(code) {
  return { code, filename: "fixture.jsx" };
}

const suite = {
  valid: [
    useCase(`
        import React from "not-react";
        React.use(Promise.resolve(42));
      `),
    useCase(`
        const React = { use: () => {} };
        React.use(Promise.resolve(42));
      `),
    useCase(`
        import { use } from "not-react";
        use(Promise.resolve(42));
      `),
    useCase(`
        function use() {}
        use(Promise.resolve(42));
      `),
    useCase(`
        import React from "react";
        function test() {
          const React = { use: () => {} };
          React.use(Promise.resolve(42));
        }
      `),
    useCase(`
        import { use } from "react";
        function test() {
          const use = () => {};
          use(Promise.resolve(42));
        }
      `),
    useCase(`
        import React from "react";
        React.use(Promise.reject(42));
        React.use(somePromise);
        React.use(Promise.all([]));
      `),
    useCase(`
        import { use } from "react";
        use(Promise.reject(42));
        use(somePromise);
        use(Promise.all([]));
      `),
    useCase(`
        import React from "react";
        React.use();
        React.use(Promise);
        React.use(Promise.resolve);
      `),
    useCase(`
        import { use } from "react";
        let localUse = use;
        function test() {
            let use = () => {};
            use(Promise.resolve(42));
        }
      `),
  ],
  invalid: [
    // 1 resolve. React.use(Promise.resolve()) imported from react.
    useCase(`
        import React from "react";
        React.use(Promise.resolve(42));
      `),
    useCase(`
        import * as React from "react";
        React.use(Promise.resolve(42));
      `),
    // 1 resolve. use imported from react.
    useCase(`
        import { use } from "react";
        use(Promise.resolve(42));
      `),
    // 1 resolve. Aliased use imported from react.
    useCase(`
        import { use as myUse } from "react";
        myUse(Promise.resolve(42));
      `),
  ],
};

describe("react-no-use-promise-resolve", () => {
  it("snapshots react-no-use-promise-resolve diagnostics", async () => {
    await expect(
      snapshotDocument({
        [rule]: snapshotSuite(rule, languageOptions, suite),
      }),
    ).toMatchFileSnapshot("snapshots/react-no-use-promise-resolve.snap");
  }, 30_000);
});
