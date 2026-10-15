import { primary, session } from '@vendor/valkey';
const opts = { client: primary };
const alias = opts;
const copied = primary.get;
const copiedObject = { get: primary.get };
const items = ['one'];
function before(client) { client.get('key'); }
for (const item of items) { before(primary); }
function replacement() {}
alias.client.get = replacement;
function scan(client) { client.get('key'); }
for (const item of items) { scan(primary); }
function untouched(client) { client.info('key'); }
for (const item of items) { untouched(primary); }
function other(client) { client.info('key'); }
for (const item of items) { other(session); }
function snapshot(get) { get('key'); }
for (const item of items) { snapshot(copied); }
function objectSnapshot(options) { options.get('key'); }
for (const item of items) { objectSnapshot(copiedObject); }
function copiedAfter(get) { get('key'); }
for (const item of items) { copiedAfter(primary.get); }

// Nested receiver paths retain sibling precision and frozen method values.
import { primary as nested } from '@vendor/valkey';
const box = { inner: { client: nested.nested } };
const nestedCopied = nested.nested.get;
box.inner.client.get = replacement;
function nestedWrite(client) { client.nested.get('key'); }
for (const item of items) { nestedWrite(nested); }
function sibling(client) { client.other.get('key'); }
for (const item of items) { sibling(nested); }
for (const item of items) { nestedCopied('key'); }

// Replacing the container's reference does not mutate the imported receiver.
import { primary as detached } from '@vendor/valkey';
const detachedBox = { client: detached };
detachedBox.client = { get: replacement };
detachedBox.client.get = replacement;
function detachedReceiver(client) { client.get('key'); }
for (const item of items) { detachedReceiver(detached); }

import { primary as dynamic } from '@vendor/valkey';
const dynamicBox = { client: dynamic };
dynamicBox.client[key] = replacement;
function dynamicWrite(client) { client.get('key'); }
for (const item of items) { dynamicWrite(dynamic); }

// Rebinding the mutable root leaves its previous receiver untouched.
import { primary as mutable } from '@vendor/valkey';
let rebound = { client: mutable };
rebound = { client: session };
rebound.client.get = replacement;
function mutableReceiver(client) { client.get('key'); }
for (const item of items) { mutableReceiver(mutable); }

// A projected receiver captured before replacement still owns its old identity.
import { primary as savedPrimary } from '@vendor/valkey';
const savedBox = { client: savedPrimary };
const saved = savedBox.client;
savedBox.client = { get: replacement };
saved.get = replacement;
function savedReceiver(client) { client.get('key'); }
for (const item of items) { savedReceiver(savedPrimary); }

// Capturing the outer object does not freeze later reads of its descendants.
import { primary as livePrimary } from '@vendor/valkey';
const liveBox = { client: livePrimary };
const outer = { box: liveBox };
liveBox.client = { get: replacement };
outer.box.client.get = replacement;
function liveReceiver(client) { client.get('key'); }
for (const item of items) { liveReceiver(livePrimary); }

import { primary as aliasPrimary } from '@vendor/valkey';
const originalBox = { client: aliasPrimary };
const sharedBox = originalBox;
sharedBox.client = { get: replacement };
originalBox.client.get = replacement;
function reboundAlias(client) { client.get('key'); }
for (const item of items) { reboundAlias(aliasPrimary); }

import { primary as calledPrimary } from '@vendor/valkey';
const calledBox = { client: calledPrimary };
function replaceCalled() { calledBox.client = { get: replacement }; }
replaceCalled();
calledBox.client.get = replacement;
function calledReceiver(client) { client.get('key'); }
for (const item of items) { calledReceiver(calledPrimary); }

import { primary as mixedPrimary } from '@vendor/valkey';
const mixedBox = { client: mixedPrimary };
mixedBox.client = { get: replacement };
function replaceMixed() { mixedBox.client = { get: replacement }; }
replaceMixed();
mixedBox.client.get = replacement;
function mixedReceiver(client) { client.get('key'); }
for (const item of items) { mixedReceiver(mixedPrimary); }

// The original receiver was already mutated before a callable rebinds the box.
import { primary as inversePrimary } from '@vendor/valkey';
const inverseBox = { client: inversePrimary };
function replaceInverse() { inverseBox.client = { get: replacement }; }
inverseBox.client.get = replacement;
replaceInverse();
function inverseReceiver(client) { client.get('key'); }
for (const item of items) { inverseReceiver(inversePrimary); }
