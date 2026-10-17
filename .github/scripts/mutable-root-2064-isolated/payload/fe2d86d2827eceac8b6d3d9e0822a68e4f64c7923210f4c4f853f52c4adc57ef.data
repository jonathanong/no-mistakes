import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
(class { @opaque method() {} });
holder.box.client.get = replacement;
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
