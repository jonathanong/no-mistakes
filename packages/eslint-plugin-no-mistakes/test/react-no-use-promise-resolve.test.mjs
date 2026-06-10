import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { messages } from "./helpers.mjs";

describe("react-no-use-promise-resolve", () => {
  it("reports React.use(Promise.resolve()) when imported from react", () => {
    assert.deepEqual(
      messages(
        `
        import React from "react";
        React.use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      ["resolve"],
    );

    assert.deepEqual(
      messages(
        `
        import * as React from "react";
        React.use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      ["resolve"],
    );
  });

  it("reports use(Promise.resolve()) when use is imported from react", () => {
    assert.deepEqual(
      messages(
        `
        import { use } from "react";
        use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      ["resolve"],
    );
  });

  it("reports aliased use(Promise.resolve()) when imported from react", () => {
    assert.deepEqual(
      messages(
        `
        import { use as myUse } from "react";
        myUse(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      ["resolve"],
    );
  });

  it("accepts React.use(Promise.resolve()) when React is NOT imported from react", () => {
    assert.deepEqual(
      messages(
        `
        import React from "not-react";
        React.use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );

    assert.deepEqual(
      messages(
        `
        const React = { use: () => {} };
        React.use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });

  it("accepts use(Promise.resolve()) when use is NOT imported from react", () => {
    assert.deepEqual(
      messages(
        `
        import { use } from "not-react";
        use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );

    assert.deepEqual(
      messages(
        `
        function use() {}
        use(Promise.resolve(42));
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });

  it("accepts shadowed variables", () => {
    assert.deepEqual(
      messages(
        `
        import React from "react";
        function test() {
          const React = { use: () => {} };
          React.use(Promise.resolve(42));
        }
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );

    assert.deepEqual(
      messages(
        `
        import { use } from "react";
        function test() {
          const use = () => {};
          use(Promise.resolve(42));
        }
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });

  it("accepts React.use with non Promise.resolve arguments", () => {
    assert.deepEqual(
      messages(
        `
        import React from "react";
        React.use(Promise.reject(42));
        React.use(somePromise);
        React.use(Promise.all([]));
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });

  it("accepts use with non Promise.resolve arguments", () => {
    assert.deepEqual(
      messages(
        `
        import { use } from "react";
        use(Promise.reject(42));
        use(somePromise);
        use(Promise.all([]));
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });

  it("handles missing object properties gracefully", () => {
    assert.deepEqual(
      messages(
        `
        import React from "react";
        React.use();
        React.use(Promise);
        React.use(Promise.resolve);
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });

  it("handles variable redefined before usage but after react import", () => {
    assert.deepEqual(
      messages(
        `
        import { use } from "react";
        let localUse = use;
        function test() {
            let use = () => {};
            use(Promise.resolve(42));
        }
      `,
        "react-no-use-promise-resolve",
      ),
      [],
    );
  });
});
