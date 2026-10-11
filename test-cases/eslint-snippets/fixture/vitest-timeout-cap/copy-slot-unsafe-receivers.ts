import { test } from "vitest";

const setter = { timeout: 30_001 };
const setterBox = {
  copy: { ...setter },
  set copy(value) { consume(value); },
};
setterBox.copy = {};
test("accessor copy slot replacement", setter, () => {});

const getter = { timeout: 30_001 };
const getterBox = {
  envelope: { copy: { ...getter } },
  get envelope() { return receiver(); },
};
delete getterBox.envelope.copy;
test("accessor receiver deletion", getter, () => {});

const getterAlias = { timeout: 30_001 };
const getterAliasBox = {
  envelope: { copy: { ...getterAlias } },
  get envelope() { return receiver(); },
};
const uncertainAlias = getterAliasBox.envelope;
delete uncertainAlias.copy;
test("accessor receiver alias deletion", getterAlias, () => {});

const inherited = { timeout: 30_001 };
const inheritedBox = { __proto__: { copy: { ...inherited } } };
inheritedBox.copy = {};
test("inherited wrapper slot", inherited, () => {});

// A different deletion can expose an inherited setter for the assignment.
const missingOwn = { timeout: 30_001 };
const missingOwnBox = { copy: { ...missingOwn } };
const missingOwnAlias = missingOwnBox;
delete missingOwnAlias.copy;
missingOwnBox.copy = {};
test("deleted own descriptor remains conservative", missingOwn, () => {});

// Ignore only the current delete, never an earlier delete of its receiver.
const deletedReceiver = { timeout: 30_001 };
const deletedReceiverBox = { envelope: { copy: { ...deletedReceiver } } };
delete deletedReceiverBox.envelope;
delete deletedReceiverBox.envelope.copy;
test("deleted receiver may expose inherited getter", deletedReceiver, () => {});

const deletedAliasReceiver = { timeout: 30_001 };
const deletedAliasReceiverBox = { envelope: { copy: { ...deletedAliasReceiver } } };
const deletedAlias = deletedAliasReceiverBox;
delete deletedAlias.envelope;
delete deletedAliasReceiverBox.envelope.copy;
test("other alias deletion invalidates receiver", deletedAliasReceiver, () => {});

const overwrittenAlias = { timeout: 30_001 };
const overwrittenAliasBox = { envelope: { copy: { ...overwrittenAlias } } };
let mutableAlias = overwrittenAliasBox.envelope;
mutableAlias = receiver();
delete mutableAlias.copy;
test("mutable receiver aliases remain conservative", overwrittenAlias, () => {});

const destructured = { timeout: 30_001 };
const destructuredBox = { envelope: { copy: { ...destructured } } };
const { envelope } = destructuredBox;
delete envelope.copy;
test("destructured receiver remains conservative", destructured, () => {});

const escaped = { timeout: 30_001 };
const escapedBox = { copy: { ...escaped } };
mutate(escapedBox);
delete escapedBox.copy;
test("escaped wrapper remains conservative", escaped, () => {});

const dynamic = { timeout: 30_001 };
const dynamicBox = { [getKey()]: { ...dynamic } };
delete dynamicBox[getKey()];
test("uncertain wrapper declaration", dynamic, () => {});

const siblings = { timeout: 30_001 };
const siblingBox = { copy: { ...siblings }, other: {} };
delete siblingBox.other;
test("unproved sibling deletion stays conservative", siblings, () => {});
