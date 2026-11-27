import { test } from "@playwright/test";

test("a", () => {
  test.setTimeout(15000);
  test.slow();
});
test("b", () => {
  test.setTimeout(15000) as void;
  test.slow();
});
test(
  "c",
  ((_: unknown, info: any) => {
    info.setTimeout(60000);
  }).bind!(null),
);
