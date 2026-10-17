import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
holder.box.client = { fn: () => { const own = { client: {} }; own.client = { x: opaque() }; } };
holder.box.client.get = replacement;
function globalDeferredOwn(value) { value.get('key'); }
globalDeferredOwn(client);
