import { expect, test } from "vitest";
import { renderWidget } from "../src/widget";
test("widget", () => {
  expect(renderWidget()).toBeDefined();
});
