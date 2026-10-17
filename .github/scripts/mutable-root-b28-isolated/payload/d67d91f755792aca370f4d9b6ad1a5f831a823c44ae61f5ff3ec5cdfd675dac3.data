import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = {};
@opaque
class Decorated {}
holder.box.client.get = replacement;
// Keep this hoisted definition after the descendant write: its placement
// otherwise activates an unrelated early global-definition history path.
function opaque() { holder.box = { client }; }
function scan(value) { value.get('key'); }
scan(client);
