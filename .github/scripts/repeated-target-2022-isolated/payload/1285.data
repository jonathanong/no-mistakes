import { primary as client } from '@vendor/valkey';
function replacement() {}
@opaque
class Decorated {}
const holder = { box: { client } };
holder.box.client = {};
holder.box.client.get = replacement;
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
