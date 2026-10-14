import { primary } from '@vendor/valkey';

function replacement() {}
// Object fields stay lazy: the first member read precedes the write, the second follows it.
const options = { client: primary };
options.client.get('before-write');
({ client: primary }).client.get('literal-before-write');
const literalCopy = ({ client: primary }).client.get;
primary.get = replacement;
options.client.get('after-write');
literalCopy('literal-copy-after-write');
({ client: primary }).client.get('inline-after-write');
