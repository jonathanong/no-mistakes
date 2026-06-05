import { utils } from "./utils.mts";
import { describe, it, expect } from "vitest";

describe("utils", () => {
  it("pick works", () => {
    expect(utils.pick({ a: 1, b: 2 }, ["a"])).toEqual({ a: 1 });
  });
});
