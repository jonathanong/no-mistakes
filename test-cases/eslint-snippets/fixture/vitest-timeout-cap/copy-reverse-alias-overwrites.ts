import { defineConfig } from "vitest/config";

// The update uses an alias; the overwrite uses the original copied binding.
const reverse = { test: { testTimeout: 1 }, counter: 2 };
const reverseCopy = { ...reverse };
const reverseAlias = reverseCopy;
reverseCopy.counter = {};
reverseAlias.counter++;
defineConfig(reverse);

// The other direction must keep rejecting object replacement as well.
const forward = { test: { testTimeout: 1 }, counter: 2 };
const forwardCopy = { ...forward };
const forwardAlias = forwardCopy;
forwardAlias.counter = {};
forwardCopy.counter += 1;
defineConfig(forward);

const sibling = { test: { testTimeout: 1 }, counter: 2 };
const siblingCopy = { ...sibling };
const siblingLeft = siblingCopy;
const siblingRight = siblingCopy;
siblingLeft.counter ||= {};
siblingRight.counter--;
defineConfig(sibling);

// Reference order cannot restore a proof after an unproved object overwrite.
const later = { test: { testTimeout: 1 }, counter: 2 };
const laterCopy = { ...later };
const laterAlias = laterCopy;
laterAlias.counter *= 2;
laterCopy.counter = {};
defineConfig(later);

const numeric = { test: { testTimeout: 1 }, counter: 2 };
const numericCopy = { ...numeric };
const numericAlias = numericCopy;
numericCopy.counter = 3;
numericAlias.counter++;
numericCopy.counter *= 2;
defineConfig(numeric);

const nullable = { test: { testTimeout: 1 }, counter: null };
const nullableCopy = { ...nullable };
const nullableAlias = nullableCopy;
nullableAlias.counter = null;
nullableCopy.counter--;
defineConfig(nullable);

const independent = { test: { testTimeout: 1 }, counter: 2, other: 0 };
const independentCopy = { ...independent };
const independentAlias = independentCopy;
independentCopy.other = {};
independentAlias.counter++;
defineConfig(independent);

// A distinct spread copy must not join the original copy's alias set.
const separate = { test: { testTimeout: 1 }, counter: 2 };
const separateCopy = { ...separate };
const secondCopy = { ...separateCopy };
secondCopy.counter = {};
separateCopy.counter++;
defineConfig(separate);

const chained = { test: { testTimeout: 1 }, counter: 2 };
const chainedCopy = { ...chained };
const chainedAlias = chainedCopy as { counter: unknown };
const chainedFinal = chainedAlias!;
chainedCopy.counter &&= {};
chainedFinal.counter++;
defineConfig(chained);

const dynamic = { test: { testTimeout: 1 }, counter: 2 };
const dynamicCopy = { ...dynamic };
const dynamicAlias = dynamicCopy;
dynamicCopy[getKey()] = {};
dynamicAlias.counter++;
defineConfig(dynamic);

const mutable = { test: { testTimeout: 1 }, counter: 2 };
const mutableCopy = { ...mutable };
mutableCopy.counter++;
let mutableAlias = mutableCopy;
mutableAlias.counter = {};
defineConfig(mutable);

const destructured = { test: { testTimeout: 1 }, counter: 2 };
const destructuredCopy = { ...destructured };
destructuredCopy.counter++;
const { counter } = destructuredCopy;
defineConfig(destructured);

const rebound = { test: { testTimeout: 1 }, counter: 2 };
const reboundCopy = { ...rebound };
reboundCopy.counter++;
const reboundAlias = reboundCopy;
reboundAlias = {};
defineConfig(rebound);

const escaped = { test: { testTimeout: 1 }, counter: 2 };
const escapedCopy = { ...escaped };
escapedCopy.counter++;
const escapedAlias = escapedCopy;
consume(escapedAlias);
defineConfig(escaped);

const keyReceiver = { test: { testTimeout: 1 }, counter: 2 };
const keyCopy = { ...keyReceiver };
keyCopy.counter++;
const unrelated = {};
unrelated[keyCopy] = 0;
defineConfig(keyReceiver);
