import testDefault from "../src/types.mjs";
import type { TestContract } from "../src/types.mjs";
const result: TestContract = { value: testDefault() };
void result;
