import { test } from "@playwright/test";

const s0 = randomSuffix();
test.beforeAll(() => {
  console.log(s0);
});
const s1 = randomSuffix() as string;
test.beforeAll(() => {
  console.log(s1);
});
const s2 = randomSuffix();
test.beforeAll((() => {
  console.log(s2);
}) as () => void);
const s3 = randomSuffix();
(test.beforeAll as typeof test.beforeAll)(() => {
  console.log(s3);
});
test("ok", (() => {
  const s4 = randomSuffix();
  test.beforeAll(() => {
    console.log(s4);
  });
}) as () => void);
test.beforeAll.apply(test, [
  (() => {
    const token = randomSuffix();
    console.log(token);
  }) as any,
] as const);
