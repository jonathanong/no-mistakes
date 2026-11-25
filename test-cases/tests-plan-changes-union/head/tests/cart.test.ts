import { expect, test } from "vitest";
import { cartTotal } from "../src/cart";
test("cart", () => {
  expect(cartTotal()).toBeGreaterThan(0);
});
