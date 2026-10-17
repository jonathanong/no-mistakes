import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
@factory()
class Decorated {}
holder.box.client.get = replacement;
function factory() { return opaque; }
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
