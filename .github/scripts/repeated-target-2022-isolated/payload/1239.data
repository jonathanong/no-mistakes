import { primary, session, primary as capturedPrimary, primary as nestedPrimary, primary as dynamicPrimary, primary as mergedPrimary } from '@vendor/valkey-imported-receiver-mutations';
const items = ['one'];
const before = { client: primary };
const copied = primary.get;
const copiedObject = { get: primary.get };
function beforeWrite({ client } = before) { client.get('key'); }
for (const item of items) { beforeWrite(); }
primary.get = replacement;
const after = { client: primary };
function lateDefault({ client } = after) { client.get('key'); }
for (const item of items) { lateDefault(); }
function earlyObject({ client } = before) { client.get('key'); }
for (const item of items) { earlyObject(); }
function suppliedObject({ client }) { client.get('key'); }
for (const item of items) { suppliedObject({ client: primary }); }
function suppliedIdentifier(client) { client.get('key'); }
for (const item of items) { suppliedIdentifier(primary); }
function lateIdentifier(client = primary) { client.get('key'); }
for (const item of items) { lateIdentifier(); }
function unrelated({ client } = after) { client.info('key'); }
for (const item of items) { unrelated({ client: session }); }
function copiedArgument(get) { get('key'); }
for (const item of items) { copiedArgument(copied); }
function copiedDefault({ get } = copiedObject) { get('key'); }
for (const item of items) { copiedDefault(); }
function copiedAfter({ get }) { get('key'); }
for (const item of items) { copiedAfter({ get: primary.get }); }
function identifierObject(options) { options.client.get('key'); }
for (const item of items) { identifierObject(after); }
function nestedSupplied({ box: { client } }) { client.get('key'); }
for (const item of items) { nestedSupplied({ box: before }); }
function unrelatedIdentifier(client = primary) { client.info('key'); }
for (const item of items) { unrelatedIdentifier(session); }
// Captured writes must invalidate receivers even when the helper appears first.
function captured({ client }) { client.get('key'); }
function mutate() { capturedPrimary.get = replacement; }
for (const item of items) { captured({ client: capturedPrimary }); }
function untouchedMethod(client) { client.info('key'); }
for (const item of items) { untouchedMethod(primary); }
const copiedInfoAfter = primary.info;
for (const item of items) { copiedInfoAfter('key'); }
for (const item of items) { primary.info('key'); }
function identifierBefore(options) { options.client.get('key'); }
for (const item of items) { identifierBefore(before); }
function forwardBefore(options) { identifierBefore(options); }
for (const item of items) { forwardBefore(before); }
function nestedIdentifier(options) { options.box.client.get('key'); }
for (const item of items) { nestedIdentifier({ box: before }); }

const nestedBefore = nestedPrimary.nested;
const nestedCopied = nestedPrimary.nested.get;
nestedPrimary.nested.get = replacement;
function nestedReceiver(client) { client.get('key'); }
for (const item of items) { nestedReceiver(nestedBefore); }
function untouchedNested(client) { client.other.get('key'); }
for (const item of items) { untouchedNested(nestedPrimary); }
for (const item of items) { nestedCopied('key'); }
dynamicPrimary[key].get = replacement;
function dynamicReceiver(client) { client.bucket.get('key'); }
for (const item of items) { dynamicReceiver(dynamicPrimary); }

const mergedAlias = mergedPrimary;
mergedAlias.get = replacement;
mergedPrimary.get = replacement;
mergedAlias[key] = replacement;
mergedPrimary[key] = replacement;
function mergedReceiver(client) { client.get('key'); }
for (const item of items) { mergedReceiver(mergedPrimary); }
