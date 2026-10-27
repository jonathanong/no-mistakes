import testOnlyDefault from "../src/test-default.mjs";
import type typedTestDefault from "../src/test-default.mjs";
export const used = [testOnlyDefault(), null as unknown as typeof typedTestDefault];
