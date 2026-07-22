import { describe, expect, it } from "vitest";

// Both projects intentionally own this file so targeted command filtering is observable.
describe("shared", () => {
  it("keeps only selected execution targets", () => expect(true).toBe(true));
});
