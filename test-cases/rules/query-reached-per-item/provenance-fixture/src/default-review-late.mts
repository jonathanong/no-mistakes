import { primary, session } from '@vendor/valkey';
const items = ['one'];
// Late declarations must see writes already visited by the extractor.
primary.get = unrelated;
function lateIdentifier(client = primary) { client.get('key'); }
for (const item of items) { lateIdentifier(); }
const lateOpts = { client: session };
lateOpts.client.info = unrelated;
function latePattern({ client } = lateOpts) { client.info('key'); }
for (const item of items) { latePattern(); }
function suppliedLate(client = primary) { client.scan('key'); }
for (const item of items) { suppliedLate(session); }
