// This test intentionally imports another test so changed-test dependents remain observable.
import "./web.test";

import { describe, expect, it } from "vitest";

describe("policy", () => {
  it("runs when its imported test changes", () => expect(true).toBe(true));
});
