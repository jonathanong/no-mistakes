import { test } from "vitest";

// This ordinary forward alias exercises the scan after a copied scalar update.
const ordinary = { timeout: 1 };
const ordinaryCopy = { ...ordinary };
ordinaryCopy.timeout += 1;
const ordinaryAlias = ordinaryCopy;
test("ordinary copied alias", ordinaryAlias, () => {});
test("ordinary source", ordinary, () => {});

// ESLint can associate an ambient declaration and a later initializer with one binding.
const repeated = { timeout: 1 };
const repeatedCopy = { ...repeated };
repeatedCopy.timeout += 1;
declare const repeatedAlias: { timeout: number };
const repeatedAlias = repeatedCopy;
test("repeated source", repeated, () => {});
