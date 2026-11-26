import { expect, test } from "vitest";

declare function loadReshapedFixture(id: string): { size: number };

test("reshaped size", () => {
  expect(loadReshapedFixture("badge").size).toBe(128);
});
