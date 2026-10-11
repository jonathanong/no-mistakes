import { test } from "vitest";
import { defineConfig } from "vitest/config";

// Replacing a copied scalar with an object invalidates a later coercion.
const overwritten = { timeout: 1, counter: 2 };
const overwrittenBox = { copy: { ...overwritten } };
overwrittenBox.copy.counter = { valueOf() { return 0; } };
overwrittenBox.copy.counter++;
test("object overwrite", overwritten, () => {});

const prefix = { timeout: 1, counter: 2 };
const prefixBox = { copy: { ...prefix } };
prefixBox.copy = { counter: {} };
prefixBox.copy.counter++;
test("receiver replacement", prefix, () => {});

const aliasSource = { timeout: 1, counter: 2 };
const aliasBox = { copy: { ...aliasSource } };
const projectedAlias = aliasBox.copy;
projectedAlias.counter = {};
aliasBox.copy.counter++;
test("projected alias overwrite", aliasSource, () => {});

const rootAliasSource = { timeout: 1, counter: 2 };
const rootAliasBox = { copy: { ...rootAliasSource } };
const rootAlias = rootAliasBox;
rootAlias.copy.counter = {};
rootAliasBox.copy.counter++;
test("root alias overwrite", rootAliasSource, () => {});

// Reverse projected aliases must not reuse the original numeric initializer.
const reverse = { timeout: 1, counter: 2 };
const reverseBox = { copy: { ...reverse } };
const reverseAlias = reverseBox.copy;
reverseBox.copy.counter = {};
reverseAlias.counter++;
test("reverse alias overwrite", reverse, () => {});

const shared = { test: { testTimeout: 1, valueOf() { this.testTimeout = 60_000; return 0; } } };
const sharedBox = { copy: { ...shared } };
sharedBox.copy.test++;
defineConfig(shared);

const primitive = { test: { testTimeout: 1, [Symbol.toPrimitive]() { this.testTimeout = 60_000; return 0; } } };
const primitiveBox = [{ ...primitive }];
primitiveBox[0].test += 1;
defineConfig(primitive);

const getter = { timeout: 1, get counter() { return 2; } };
const getterBox = { copy: { ...getter } };
getterBox.copy.counter++;
test("copied getter", getter, () => {});

const callable = { timeout: 1, counter: () => 2 };
const callableBox = { copy: { ...callable } };
callableBox.copy.counter++;
test("copied callback", callable, () => {});

const makeCounter = () => null;
const aliasedCallable = { timeout: 1, counter: makeCounter };
const callableArray = [{ ...aliasedCallable }];
callableArray[0].counter++;
test("aliased callable marker", aliasedCallable, () => {});

const opaque = { timeout: 1, counter: 2 };
const opaqueBox = { copy: { ...opaque, ...getOptions() } };
opaqueBox.copy.counter++;
test("opaque copied spread", opaque, () => {});

const cyclic = { ...cyclic, timeout: 1, counter: 2 };
const cycleBox = { copy: { ...cyclic } };
cycleBox.copy.counter++;
test("cyclic copied spread", cyclic, () => {});

const dynamic = { timeout: 1, counter: 2 };
const dynamicBox = { copy: { ...dynamic } };
dynamicBox[getKey()].counter++;
test("uncertain receiver key", dynamic, () => {});

// A sibling getter can change the copied counter using this rather than a binding reference.
const sibling = { timeout: 1, counter: 2 };
const siblingBox = { copy: { ...sibling }, get trigger() { this.copy.counter = {}; return {}; } };
defineConfig(siblingBox.trigger);
siblingBox.copy.counter++;
test("sibling getter", sibling, () => {});

const inherited = { timeout: 1, counter: 2 };
const inheritedBox = { copy: { ...inherited }, __proto__: { get trigger() { this.copy.counter = {}; return {}; } } };
defineConfig(inheritedBox.trigger);
inheritedBox.copy.counter++;
test("prototype getter", inherited, () => {});

const literalAlias = { timeout: 1, counter: 2 };
const innerCopy = { ...literalAlias };
const identifierBox = { copy: innerCopy };
innerCopy.counter = {};
identifierBox.copy.counter++;
test("identifier projected receiver", literalAlias, () => {});

const unknownWrite = { timeout: 1, counter: 2 };
const unknownWriteBox = { copy: { ...unknownWrite } };
unknownWriteBox.copy[getKey()] = {};
unknownWriteBox.copy.counter++;
test("unknown overwrite key", unknownWrite, () => {});

// A later replacement still invalidates the scalar proof across the whole binding.
const laterUnknown = { timeout: 1, counter: 2 };
const laterUnknownBox = { copy: { ...laterUnknown } };
laterUnknownBox.copy.counter++;
laterUnknownBox.copy[getKey()] = {};
test("later unknown overwrite key", laterUnknown, () => {});

const arraySpread = { timeout: 1, counter: 2 };
const arraySpreadBox = [{ ...arraySpread }, ...[]];
arraySpreadBox[0].counter++;
test("array spread wrapper", arraySpread, () => {});

const shadowed = { timeout: 1, counter: 2 };
const shadowedBox = { outer: { copy: { ...shadowed } }, outer: 2 };
shadowedBox.outer.copy.counter++;
test("nonliteral shadowed ancestor", shadowed, () => {});

const missing = { timeout: 1, counter: 2 };
const missingBox = { outer: { copy: { ...missing } }, outer: { other: 0 } };
missingBox.outer.copy.counter++;
test("missing shadowed ancestor", missing, () => {});

const computedWrapper = { timeout: 1, counter: 2 };
const computedWrapperBox = { copy: { ...computedWrapper }, [getKey()]: {} };
computedWrapperBox.copy.counter++;
test("unknown wrapper property", computedWrapper, () => {});

const spreadWrapper = { timeout: 1, counter: 2 };
const spreadWrapperBox = { ...{ copy: { ...spreadWrapper } } };
spreadWrapperBox.copy.counter++;
test("object spread wrapper", spreadWrapper, () => {});

const prototypeSource = { timeout: 1, counter: 2, __proto__: {} };
const prototypeSourceBox = { copy: { ...prototypeSource } };
prototypeSourceBox.copy.counter++;
test("prototype copied source", prototypeSource, () => {});

const computedSource = { timeout: 1, counter: 2, [getKey()]: {} };
const computedSourceBox = { copy: { ...computedSource } };
computedSourceBox.copy.counter++;
test("unknown copied source property", computedSource, () => {});

const readEscape = { timeout: 1, counter: 2 };
const readEscapeBox = { copy: { ...readEscape } };
readEscapeBox.copy.counter++;
const sink = {};
sink.value = readEscapeBox.copy.counter;
test("copied scalar read escapes", readEscape, () => {});

const asserted = { timeout: 1, counter: 2 };
const assertedBox = { copy: { ...asserted } };
(assertedBox.copy.counter as unknown) = {};
assertedBox.copy.counter++;
test("asserted object overwrite", asserted, () => {});

// Inline literal receivers must reject safely rather than dereferencing a missing binding.
const inline = { timeout: 1, counter: 2 };
({ copy: { ...inline } }).copy.counter++;
test("inline object receiver", inline, () => {});

const inlineArray = { timeout: 1, counter: 2 };
[{ ...inlineArray }][0].counter++;
test("inline array receiver", inlineArray, () => {});

const callReceiver = { timeout: 1, counter: 2 };
getBox({ copy: { ...callReceiver } }).copy.counter++;
test("call receiver", callReceiver, () => {});

const unboundReceiver = { timeout: 1, counter: 2 };
unboundBox = { copy: { ...unboundReceiver } };
unboundBox.copy.counter++;
test("unbound receiver", unboundReceiver, () => {});

// Noncanonical numeric strings name ordinary array properties rather than slots.
const noncanonical = { timeout: 1, counter: 2 };
const noncanonicalBox = [{ ...noncanonical }];
noncanonicalBox["00"].counter++;
test("noncanonical array property", noncanonical, () => {});

// The direct-binding ordering exception does not prove wrapped receiver writes.
const futureOverwrite = { timeout: 1, counter: 2 };
const futureOverwriteBox = { copy: { ...futureOverwrite } };
futureOverwriteBox.copy.counter++;
futureOverwriteBox.copy.counter = {};
test("later wrapped object overwrite", futureOverwrite, () => {});
