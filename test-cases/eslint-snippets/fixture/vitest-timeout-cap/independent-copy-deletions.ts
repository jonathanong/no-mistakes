import { defineConfig } from 'vitest/config';
import { test } from 'vitest';

const independent = { timeout: 30_001 };
const first = { ...independent };
delete first.timeout;
const second = { ...independent };
second.timeout = 2;
test('independent copies', () => {}, independent);

const aliased = { timeout: 1 };
const removed = { ...aliased };
const removedAlias = removed;
delete removedAlias.timeout;
const remaining = { ...aliased };
remaining.timeout = 2;
test('independent alias', () => {}, aliased);

const wrapped = { timeout: 30_001 };
const wrapper = { copy: { ...wrapped } };
delete wrapper.copy.timeout;
const otherWrapper = { copy: { ...wrapped } };
otherWrapper.copy.timeout = 2;
test('independent wrappers', () => {}, wrapped);

const shared = { test: { testTimeout: 1 } };
const sharedCopy = { ...shared, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 };
const box = { copy: sharedCopy };
const boxCopy = { ...box };
delete boxCopy.copy.deadline;
sharedCopy.deadline = 30_001;
defineConfig(shared);

const projected = { test: { testTimeout: 1 } };
const originalBox = { copy: { ...projected, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 } };
const deletingBox = { ...originalBox };
delete deletingBox.copy.deadline;
const writingBox = { ...originalBox };
writingBox.copy.deadline = 30_001;
defineConfig(projected);

const arrayShared = { test: { testTimeout: 1 } };
const array = [{ ...arrayShared, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 }];
const deletingArray = [...array];
delete deletingArray[0].deadline;
const writingArray = [...array];
writingArray[0].deadline = 30_001;
defineConfig(arrayShared);

const memberAlias = { test: { testTimeout: 1 } };
const aliasBox = { copy: { ...memberAlias, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 } };
const aliasBoxCopy = { ...aliasBox };
const childAlias = aliasBoxCopy.copy;
delete childAlias.deadline;
aliasBox.copy.deadline = 30_001;
defineConfig(memberAlias);

const directlyDeleted = { timeout: 1 };
delete directlyDeleted.timeout;
const directCopy = { ...directlyDeleted };
directCopy.timeout = 2;
test('direct deletion stays unknown', () => {}, directlyDeleted);

const arrayIndependent = { timeout: 30_001 };
const sourceArray = [arrayIndependent];
const removedArray = [...sourceArray];
delete removedArray[0];
const remainingArray = [...sourceArray];
remainingArray[0] = { timeout: 2 };
test('independent array slot deletion', () => {}, arrayIndependent);

const uncertain = { timeout: 1 };
let uncertainCopy = { ...uncertain };
delete uncertainCopy.timeout;
const certainCopy = { ...uncertain };
certainCopy.timeout = 2;
test('mutable copied identity stays unknown', () => {}, uncertain);

const deeplyProjected = { test: { testTimeout: 1 } };
// The multi-member receiver must retain the original container identity.
const outer = { envelope: { copy: { ...deeplyProjected, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 } } };
const envelopeCopy = { ...outer.envelope };
delete envelopeCopy.copy.deadline;
outer.envelope.copy.deadline = 30_001;
defineConfig(deeplyProjected);

const aliasProjected = { test: { testTimeout: 1 } };
const aliasOuter = { envelope: { copy: { ...aliasProjected, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 } } };
const aliasEnvelopeCopy = { ...aliasOuter.envelope };
delete aliasEnvelopeCopy.copy.deadline;
const outerAlias = aliasOuter;
outerAlias.envelope.copy.deadline = 30_001;
defineConfig(aliasProjected);

const copiedOuterSource = { test: { testTimeout: 1 } };
const copiedOuter = { envelope: { copy: { ...copiedOuterSource, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 } } };
const copiedEnvelope = { ...copiedOuter.envelope };
delete copiedEnvelope.copy.deadline;
const copiedOuterAlias = { ...copiedOuter };
copiedOuterAlias.envelope.copy.deadline = 30_001;
defineConfig(copiedOuterSource);

const memberOuterSource = { test: { testTimeout: 1 } };
const memberOuter = { envelope: { copy: { ...memberOuterSource, __proto__: { set deadline(value: number) { this.test.testTimeout = value; } }, deadline: 0 } } };
const memberEnvelopeCopy = { ...memberOuter.envelope };
delete memberEnvelopeCopy.copy.deadline;
const memberOuterAlias = { envelope: memberOuter.envelope };
memberOuterAlias.envelope.copy.deadline = 30_001;
defineConfig(memberOuterSource);
