import { test as base } from "@playwright/test";
import { makeTest, makeProvenTest, wrapTest } from "../helpers/index.mjs";

const maybe = makeTest();
maybe.describe.configure({ retries: 0 });

const proven = makeProvenTest();
proven.describe.configure({ retries: 0 });

const wrapped = makeTest(base);
wrapped.describe.configure({ retries: 0 });

const transparent = wrapTest(base);
transparent.describe.configure({ retries: 0 });
