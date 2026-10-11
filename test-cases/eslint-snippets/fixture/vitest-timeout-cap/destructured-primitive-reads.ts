import { test } from "vitest";

// The initializer's outer timeout is not the timeout of the destructured binding.
const { options: oversized } = { timeout: 1, options: { timeout: 30_001 } };
consume(oversized.timeout);
test("nested oversized timeout", oversized, () => {});

const { options: bounded } = { timeout: 30_001, options: { timeout: 1 } };
consume(bounded.timeout);
test("nested bounded timeout", bounded, () => {});

const { options: unresolved } = { timeout: 1, options: { timeout: dynamic } };
consume(unresolved.timeout);
test("nested unresolved timeout", unresolved, () => {});

const { options: aliased } = { timeout: 1, options: { timeout: dynamic } };
const alias = aliased;
consume(alias.timeout);
test("destructured alias", alias, () => {});

// Identifier declarators retain primitive reads and stable object aliases.
const ordinary = { timeout: 1 };
consume(ordinary.timeout);
test("ordinary identifier", ordinary, () => {});
const ordinaryAlias = ordinary;
test("ordinary alias", ordinaryAlias, () => {});
const ordinaryHigh = { timeout: 30_001 };
consume(ordinaryHigh.timeout);
test("ordinary oversized identifier", ordinaryHigh, () => {});
