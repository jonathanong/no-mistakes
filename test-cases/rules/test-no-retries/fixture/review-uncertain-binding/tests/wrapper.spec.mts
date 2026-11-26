import { test as base } from "@playwright/test";
import { customTest } from "../helpers/fixture.mjs";
import { choose } from "../helpers/wrappers.mjs";

const wrapped = choose(base);
wrapped.describe.configure({ retries: 0 });

const wrappedCustom = choose(customTest);
wrappedCustom.describe.configure({ retries: 2, mode: "serial" });
