import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
const alias = holder;
alias.box.client = { x: (holder.box = { client }) };
alias.box.client.get = replacement;
function globalDirectAlias(value) { value.get('key'); }
globalDirectAlias(client);
