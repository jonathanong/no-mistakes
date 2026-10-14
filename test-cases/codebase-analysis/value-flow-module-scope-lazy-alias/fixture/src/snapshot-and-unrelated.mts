import { primary } from '@vendor/valkey';

function replacement() {}
const copied = primary.get;
const options = { client: primary };
primary.get = replacement;
// The copied method is a snapshot, and a write to get does not invalidate info.
copied('copied-snapshot');
options.client.info('unrelated-method');
