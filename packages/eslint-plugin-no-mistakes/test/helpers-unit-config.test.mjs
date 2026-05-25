import { describe, it, expect } from "vitest";

import helpers from "../src/helpers.js";

const { canonicalAttribute, isSelectorAttribute, options, selectorAttributes } = helpers;

describe("helpers", () => {
  describe("options", () => {
    it("returns context.options[0] if it exists", () => {
      const context = { options: [{ foo: "bar" }] };
      expect(options(context)).toEqual({ foo: "bar" });
    });

    it("returns an empty object if options[0] is undefined", () => {
      const context = { options: [] };
      expect(options(context)).toEqual({});
    });
  });

  describe("selectorAttributes", () => {
    it("returns selectorAttributes from option if it exists", () => {
      const option = { selectorAttributes: ["data-test"] };
      expect(selectorAttributes(option)).toEqual(["data-test"]);
    });

    it("returns DEFAULT_SELECTOR_ATTRIBUTES if missing from option", () => {
      const option = {};
      expect(selectorAttributes(option)).toEqual(["data-testid", "data-pw"]);
    });
  });

  describe("canonicalAttribute", () => {
    it("returns canonicalAttribute from option if it exists", () => {
      const option = { canonicalAttribute: "data-test" };
      expect(canonicalAttribute(option)).toEqual("data-test");
    });

    it("returns 'data-pw' if missing from option", () => {
      const option = {};
      expect(canonicalAttribute(option)).toEqual("data-pw");
    });
  });

  describe("isSelectorAttribute", () => {
    it("returns true if attribute is in array", () => {
      expect(isSelectorAttribute("data-testid", ["data-testid", "data-pw"])).toBe(true);
    });

    it("returns false if attribute is not in array", () => {
      expect(isSelectorAttribute("id", ["data-testid", "data-pw"])).toBe(false);
    });
  });
});
