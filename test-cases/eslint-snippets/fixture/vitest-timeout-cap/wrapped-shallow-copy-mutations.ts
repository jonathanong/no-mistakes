import { test } from "vitest";
import { defineConfig } from "vitest/config";
const nested = { test: { testTimeout: 1 } };
const nestedBox = { copy: { ...nested } };
nestedBox.copy.test.testTimeout = 30_001;
defineConfig(nested);
const extracted = { test: { testTimeout: 1 } };
const extractedBox = { copy: { ...extracted } };
const extractedAlias = extractedBox.copy.test;
extractedAlias.testTimeout++;
defineConfig(extracted);
const array = { test: { testTimeout: 1 } };
const arrayBox = [{ ...array }];
arrayBox[0].test.testTimeout = 30_001;
defineConfig(array);
const unknownKey = { timeout: 1 };
const unknownBox = { [dynamicKey]: { ...unknownKey } };
unknownBox[dynamicKey].timeout = 30_001;
test("uncertain wrapper key", unknownKey, () => {});
const unknownIndex = { timeout: 1 };
const unknownArray = [...entries, { ...unknownIndex }];
unknownArray[1].timeout = 30_001;
test("uncertain array offset", unknownIndex, () => {});
const escaped = { timeout: 1 };
const escapedBox = { copy: { ...escaped } };
mutate(escapedBox.copy);
test("wrapped helper escape", escaped, () => {});
const uncertain = { timeout: 1 };
const uncertainBox = { copy: { ...uncertain } };
const uncertainAlias = condition ? uncertainBox.copy : uncertain;
uncertainAlias.timeout = 30_001;
test("uncertain wrapped alias", uncertain, () => {});
const readonly = { timeout: 1 };
const readonlyBox = { [dynamicKey]: { ...readonly } };
test("unmodified uncertain wrapper", readonly, () => {});
const sibling = { timeout: 1 };
const siblingBox = { copy: { ...sibling }, shared: sibling };
siblingBox.shared.timeout = 30_001;
test("shared sibling slot", sibling, () => {});
const siblingAliasSource = { timeout: 1 };
const siblingAliasBox = { copy: { ...siblingAliasSource }, shared: siblingAliasSource };
const siblingAlias = siblingAliasBox.shared;
siblingAlias.timeout = 30_001;
test("shared sibling alias", siblingAliasSource, () => {});
// A numeric object key need not correspond to a custom iterable's yielded value.
const iterable = { test: { testTimeout: 1 } };
const iterableBox = {
  0: { ...iterable },
  *[Symbol.iterator]() { yield this[0].test; },
};
const fromIterable = [...iterableBox];
fromIterable[0].testTimeout = 30_001;
defineConfig(iterable);
const objectIterable = { test: { testTimeout: 1 } };
const beforeObject = [{ ...objectIterable }];
const asObject = {
  ...beforeObject,
  *[Symbol.iterator]() { yield this[0].test; },
};
const afterObject = [...asObject];
afterObject[0].testTimeout = 30_001;
defineConfig(objectIterable);
const unknownSpreadIndex = { timeout: 1 };
const knownArray = [{ ...unknownSpreadIndex }];
const unknownSpreadArray = [...entries, ...knownArray];
unknownSpreadArray[1].timeout = 30_001;
test("uncertain spread-wrapper offset", unknownSpreadIndex, () => {});
