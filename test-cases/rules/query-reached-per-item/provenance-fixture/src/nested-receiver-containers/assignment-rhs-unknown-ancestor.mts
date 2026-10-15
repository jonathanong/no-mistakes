import { primary, replica } from '@vendor/valkey';
function replacement() {}
function unknown() { return globalThis.external; }
const box = { wrapper: { client: primary } };
box.wrapper.client = replica;
box.wrapper = unknown();
// An unknown ancestor may still denote the object with the earlier child store.
box.wrapper.client.get = replacement;

function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
