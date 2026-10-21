import test from "@playwright/test";
import * as pw from "@playwright/test";

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
