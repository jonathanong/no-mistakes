import { test } from "vitest";
import { defineConfig } from "vitest/config";

const bounded = { timeout: 1 };
const boundedBox = { copy: { ...bounded } };
boundedBox.copy = {};
test("replaced copy leaves source bounded", bounded, () => {});

const oversized = { timeout: 30_001 };
const oversizedBox = { copy: { ...oversized } };
oversizedBox.copy = { timeout: 1 };
test("replaced copy leaves source oversized", oversized, () => {});

const deleted = { timeout: 30_001 };
const deletedBox = { copy: { ...deleted } };
delete deletedBox.copy;
test("deleted copy leaves source oversized", deleted, () => {});

const ancestor = { test: { testTimeout: 30_001 } };
const ancestorBox = { envelope: { copy: { ...ancestor } } };
ancestorBox.envelope = {};
defineConfig(ancestor);

const nestedDelete = { test: { testTimeout: 30_001 } };
const nestedDeleteBox = { envelope: { copy: { ...nestedDelete } } };
delete nestedDeleteBox.envelope.copy;
defineConfig(nestedDelete);

const deletedAncestor = { timeout: 1 };
const deletedAncestorBox = { envelope: { copy: { ...deletedAncestor } } };
delete deletedAncestorBox.envelope;
test("deleted ancestor leaves source bounded", deletedAncestor, () => {});

const aliased = { timeout: 30_001 };
const aliasBox = { envelope: { copy: { ...aliased } } };
const boxAlias = aliasBox;
const envelopeAlias = boxAlias.envelope;
envelopeAlias.copy = {};
test("partial wrapper alias replacement", aliased, () => {});

const aliasDelete = { timeout: 30_001 };
const aliasDeleteBox = { envelope: { copy: { ...aliasDelete } } };
const aliasDeleteReceiver = aliasDeleteBox.envelope;
delete aliasDeleteReceiver.copy;
test("partial wrapper alias deletion", aliasDelete, () => {});

const array = { timeout: 30_001 };
const arrayBox = [{ ...array }];
arrayBox[0] = {};
test("array slot replacement", array, () => {});

const nestedArray = { timeout: 30_001 };
const nestedArrayBox = { entries: [null, { copy: { ...nestedArray } }] };
delete nestedArrayBox.entries[1].copy;
test("nested array slot deletion", nestedArray, () => {});

const arrayAncestor = { timeout: 30_001 };
const arrayAncestorBox = { entries: [{ ...arrayAncestor }] };
arrayAncestorBox.entries = [];
test("array ancestor replacement", arrayAncestor, () => {});

const computed = { timeout: 30_001 };
const slot = "copy";
const computedBox = { [slot]: { ...computed } };
computedBox[slot] = {};
test("known computed slot replacement", computed, () => {});

const shared = { test: { testTimeout: 30_001 } };
const sharedBox = { copy: { ...shared } };
sharedBox.copy.test.testTimeout = 1;
defineConfig(shared);

// Assignment RHS references must still invalidate the shared source independently.
const exposed = { timeout: 30_001 };
const exposedBox = { copy: { ...exposed } };
exposedBox.copy = expose(exposed);
test("replacement RHS escapes source", exposed, () => {});

const restored = { timeout: 30_001 };
const restoredBox = { copy: { ...restored } };
restoredBox.copy = restored;
restoredBox.copy.timeout = 1;
test("source identity restored on RHS", restored, () => {});

const dynamic = { timeout: 30_001 };
const dynamicBox = { copy: { ...dynamic } };
dynamicBox[getKey()] = {};
test("unknown replacement key", dynamic, () => {});

const offset = { timeout: 30_001 };
const offsetBox = [...unknownEntries, { ...offset }];
offsetBox[1] = {};
test("unknown array offset replacement", offset, () => {});

const coercing = { test: { testTimeout: 30_001 } };
const coercingBox = { copy: { ...coercing } };
coercingBox.copy++;
defineConfig(coercing);

const compound = { timeout: 30_001 };
const compoundBox = { envelope: { copy: { ...compound } } };
compoundBox.envelope += 1;
test("ancestor coercion remains unknown", compound, () => {});

const logical = { timeout: 30_001 };
const logicalBox = { copy: { ...logical } };
logicalBox.copy ||= {};
test("ancestor logical assignment remains conservative", logical, () => {});
