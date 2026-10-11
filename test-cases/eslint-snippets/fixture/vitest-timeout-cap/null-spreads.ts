import { defineConfig } from "vitest/config";

const nullAfterSource = { test: { testTimeout: 1 }, counter: 0 };
const nullAfter = { ...nullAfterSource, ...null };
nullAfter.counter++;
defineConfig(nullAfterSource);

const nullAliasSource = { test: { testTimeout: 1 }, counter: 0 };
const nullAlias = null;
const nullAliasAfter = { ...nullAliasSource, ...nullAlias };
nullAliasAfter.counter++;
defineConfig(nullAliasSource);

const nullBeforeSource = { test: { testTimeout: 1 }, counter: 0 };
const nullBefore = { ...null, ...nullBeforeSource };
nullBefore.counter++;
defineConfig(nullBeforeSource);

const wrappedNullSource = { test: { testTimeout: 1 }, counter: 0 };
const wrappedNull = null as null;
const wrappedNullAfter = { ...wrappedNullSource, ...(wrappedNull satisfies null) };
wrappedNullAfter.counter++;
defineConfig(wrappedNullSource);

const mutableNullSource = { test: { testTimeout: 1 }, counter: 0 };
let mutableNull: null | object = null;
const mutableNullSpread = { ...mutableNullSource, ...mutableNull };
mutableNullSpread.counter++;
defineConfig(mutableNullSource);
mutableNull = {};

const opaqueSpreadSource = { test: { testTimeout: 1 }, counter: 0 };
const opaqueSpreadControl = { ...opaqueSpreadSource, ...getOpaqueSpread() };
opaqueSpreadControl.counter++;
defineConfig(opaqueSpreadSource);

const overwrittenSource = { test: { testTimeout: 1 }, counter: 0 };
const overwrittenWithObject = { ...overwrittenSource, counter: {} };
overwrittenWithObject.counter++;
defineConfig(overwrittenSource);
