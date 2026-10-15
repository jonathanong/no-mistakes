import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { klass: class { static field = opaque(); } };
holder.box.client.get = replacement;
function globalStaticCall(value) { value.get('key'); }
globalStaticCall(client);
