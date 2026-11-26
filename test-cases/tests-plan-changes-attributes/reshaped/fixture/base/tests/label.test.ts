import { expect, test } from "vitest";

// Loader reshapes raw.size into label, so attribute narrowing must stay off unless opted in.
declare function loadReshapedFixture(id: string): { label: string };

test("reshaped label from raw size", () => {
  expect(loadReshapedFixture("badge").label).toBe("128");
});
