import { test } from "vitest";

const numeric = { timeout: 1, counter: 2 };
const numericBox = { copy: { ...numeric } };
numericBox.copy.counter++;
numericBox.copy.counter *= 2;
test("object wrapped scalar", numeric, () => {});

const nullable = { timeout: 1, counter: null };
const nullableBox = [{ ...nullable }];
nullableBox[0].counter!--;
test("array wrapped null", nullable, () => {});

const deep = { timeout: 1, counter: 2 };
const deepBox = { envelope: [, { copy: { ...deep } }] };
(deepBox.envelope[1].copy.counter as number) += 1;
test("deep sparse wrapper", deep, () => {});

const key = "copy";
const computed = { timeout: 1, counter: 2 };
const computedBox = { [key]: { ...null, ...computed, ...null } };
computedBox[key].counter--;
test("known wrapper key", computed, () => {});

const replacement = { timeout: 1, counter: 2, other: 0 };
const replacementBox = { copy: { ...replacement } };
replacementBox.copy.counter = 3;
replacementBox.copy.other = {};
replacementBox.copy.counter++;
test("scalar replacement", replacement, () => {});

const high = { timeout: 30_001, counter: 2 };
const highBox = { copy: { ...high } };
highBox.copy.counter++;
test("original remains oversized", high, () => {});

const invalid = { timeout: 0, counter: null };
const invalidBox = [{ ...invalid }];
invalidBox[0].counter++;
test("original remains invalid", invalid, () => {});
