import test from "@playwright/test";
import * as pw from "@playwright/test";

const randomSuffix = crypto.randomUUID;
const suffix = randomSuffix();
const mint = randomSuffix;
const other = mint();

test["beforeAll"](async () => {
  await createPost({ slug: suffix });
});
pw.test.beforeAll(async () => {
  await createPost({ slug: other });
});
(test?.beforeAll)(async () => {
  await createPost({ slug: suffix });
});
test.beforeAll.apply(test, [
  async () => {
    const fresh = randomSuffix();
    await createPost({ slug: fresh });
  },
]);
