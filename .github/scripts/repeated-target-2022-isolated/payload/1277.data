import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { klass: class { [opaque()]() {} } };
holder.box.client.get = replacement;
function globalComputedCall(value) { value.get('key'); }
globalComputedCall(client);
