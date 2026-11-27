import { expect, test } from "vitest";
import { message } from "./message";

test("returns the greeting", () => {
  expect(message).toBe("hello");
});
