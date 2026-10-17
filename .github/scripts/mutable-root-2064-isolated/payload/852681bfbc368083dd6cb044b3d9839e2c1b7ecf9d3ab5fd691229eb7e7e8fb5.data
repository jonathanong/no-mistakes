import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
const own = { client: {} };
holder.box.client = { klass: class { field = (own.client = { x: opaque() }); } };
holder.box.client.get = replacement;
function globalInstanceOwn(value) { value.get('key'); }
globalInstanceOwn(client);
