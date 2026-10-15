import { primary, session } from '@vendor/valkey';
const box = { client: primary };
const alias = box;
const receiver = primary;
const copied = primary.get;
alias.client.get = replacement;
alias.client.info = replacement;
const deep = { inner: { client: primary.nested } };
deep.inner.client.get = replacement;
function scan(client = primary) { client.get('key'); }
