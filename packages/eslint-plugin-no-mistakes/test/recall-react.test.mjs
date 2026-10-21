import assert from "node:assert/strict";
import { describe, it } from "vitest";
import { fixture, messages } from "./helpers.mjs";

const snippet = (name) => fixture(name);

describe("react same-file recall", () => {
  it("reports immediately invoked functions reached through forwarding", () => {
    assert.deepEqual(messages(snippet("react-no-iife-in-jsx/recall.tsx"), "react-no-iife-in-jsx"), [
      "iife",
      "iife",
      "iife",
      "iife",
    ]);
  });

  it("reports nullish react nodes reached through static keys and const aliases", () => {
    assert.equal(
      messages(
        snippet("react-no-nullish-react-node/recall.tsx"),
        "react-no-nullish-react-node",
        undefined,
        "recall.tsx",
      ).length,
      4,
    );
  });

  it("reports React.use(Promise.resolve) through the same-file spellings", () => {
    assert.equal(
      messages(
        snippet("react-no-use-promise-resolve/recall.tsx"),
        "react-no-use-promise-resolve",
        undefined,
        "recall.tsx",
      ).length,
      10,
    );
  });
});
