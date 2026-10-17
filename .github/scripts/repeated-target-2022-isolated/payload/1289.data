import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
// A copied callable keeps the target acquired before decorator application.
const saved = client.get;
@opaque
class Decorated {}
holder.box.client.get = replacement;
function opaque() { holder.box = { client }; }
function scan(value) { saved('key'); }
scan(client);
