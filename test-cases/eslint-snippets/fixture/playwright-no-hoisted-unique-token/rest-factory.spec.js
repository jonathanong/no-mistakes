import { test } from "@playwright/test";
// The rest binding is not a statically selected token factory.
const { other, ...rest } = factories;
const suffix = rest();
test.beforeAll(() => use(suffix));
