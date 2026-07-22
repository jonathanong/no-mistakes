import { describe, expect, it } from "vitest";
import { webValue } from "./helper";

describe("web", () => {
  it("is not selected for narrow database changes", () => expect(webValue).toBe(true));
});
