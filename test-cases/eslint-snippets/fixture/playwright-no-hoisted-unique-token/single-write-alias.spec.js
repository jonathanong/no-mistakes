import { test as base } from "@playwright/test";
import * as pw from "@playwright/test";

let test = base;
let runner = pw;
let mint = randomSuffix;
const suffix = mint();
let memberFactory = helpers.randomSuffix;
const fromMember = memberFactory();
let { randomSuffix: other } = helpers;
const fromOther = other();

test.beforeAll(async () => {
  await createPost({ slug: suffix });
});
runner.test.beforeAll(async () => {
  await createPost({ slug: fromMember });
  await createPost({ slug: fromOther });
});

// Reassignment fails closed: this hook is not the imported test.
let shadowed = base;
shadowed = otherTest;
const ignored = randomSuffix();
shadowed.beforeAll(async () => {
  await createPost({ slug: ignored });
});
