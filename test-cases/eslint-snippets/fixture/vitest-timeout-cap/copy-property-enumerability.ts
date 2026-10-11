import { test } from "vitest";
// Array length is own data, but object spread does not copy non-enumerable fields.
const missing = { timeout: 1 };
const missingArray = [missing];
const missingCopy = {
  ...missingArray,
  __proto__: { set length(value: number) { this[0].timeout = value; } },
};
missingCopy.length = 30_001;
test("inherited setter for unspread array length", missing, () => {});
const oversized = { timeout: 30_001 };
const oversizedArray = [oversized];
const oversizedCopy = {
  ...oversizedArray,
  __proto__: { set length(value: number) { this[0].timeout = value; } },
};
oversizedCopy.length = 1;
test("length setter can replace oversized shared timeout", oversized, () => {});
const accessor = { timeout: 1 };
const accessorArray = [accessor];
const accessorCopy = {
  set length(value: number) { this[0].timeout = value; },
  ...accessorArray,
};
accessorCopy.length = 30_001;
test("array spread does not override own length setter", accessor, () => {});
const direct = { timeout: 1 };
const directArray = [direct];
const directCopy = [...directArray];
directCopy.length = 0;
test("direct array copy length is own data", direct, () => {});
const indexed = { timeout: 1 };
const indexedArray = [indexed];
const indexedCopy = { ...indexedArray };
indexedCopy[0] = { timeout: 30_001 };
test("enumerable array index copies independently", indexed, () => {});
const explicit = { timeout: 1 };
const explicitArray = [explicit];
const explicitCopy = {
  ...explicitArray,
  __proto__: { set length(value: number) { this[0].timeout = value; } },
  length: 0,
};
explicitCopy.length = 1;
test("explicit own length overrides inherited setter", explicit, () => {});
