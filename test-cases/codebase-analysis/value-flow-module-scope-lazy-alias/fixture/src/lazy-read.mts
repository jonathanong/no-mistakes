import { primary } from '@vendor/valkey';

function replacement() {}
// Object fields stay lazy: the first member read precedes the write, the second follows it.
const options = { client: primary };
options.client.get('before-write');
primary.get = replacement;
options.client.get('after-write');
