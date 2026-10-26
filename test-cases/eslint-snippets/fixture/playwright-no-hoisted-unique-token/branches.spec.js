import { test as base } from "@playwright/test";
import * as pw from "@playwright/test";

const test = base;

const runner = pw;
const outer = pw;
function registerNamespace() {
  const pw = outer;
  pw.test.beforeAll(() => {
    use(suffix);
  });
}
const alias = test;
// Reassignment fails closed: loose is not the namespace, and mint is not the factory.
let loose = pw;
loose = unbound;
var twice;
var twice = test;

const suffix = helpers.randomSuffix();
const nested = randomSuffix()();
let mint = randomSuffix;
mint = other;
const fromLet = mint();
const { other: skip, randomSuffix: make = fallback, ...rest } = helpers;
const fromMake = make();
const dynamic = "randomSuffix";
const { [dynamic]: hidden } = helpers;
const fromHidden = hidden();
const [first] = [randomSuffix];
const fromFirst = first();
randomSuffix.call(...args);
randomSuffix.apply(null, ["a"]?.concat());

test.beforeAll(async () => {
  await createPost({ slug: suffix });
});
runner.test.beforeAll(async () => {
  await createPost({ slug: suffix });
});
const hook = test;
hook.beforeAll(async () => {
  await createPost({ slug: suffix });
});
mystery.test.beforeAll(() => {
  use(suffix);
});
alias.beforeAll(async () => {
  await createPost({ slug: suffix });
});
loose.test.beforeAll(async () => {
  await createPost({ slug: suffix });
});
twice.beforeAll(async () => {
  await createPost({ slug: suffix });
});
void fromLet;
void fromMake;
void fromHidden;
void fromFirst;
void nested;
void skip;
void rest;
