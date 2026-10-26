import testDefault from "../src/types.mjs";
// Multiline specifiers must keep their explicit named type selector.
import type {
  TestContract,
} from "../src/types.mjs";
const result: TestContract = { value: testDefault() };
void result;
