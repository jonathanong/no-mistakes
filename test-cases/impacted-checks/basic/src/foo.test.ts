import { foo } from "./foo";

test("foo increments", () => {
  if (foo(1) !== 2) {
    throw new Error("bad");
  }
});
