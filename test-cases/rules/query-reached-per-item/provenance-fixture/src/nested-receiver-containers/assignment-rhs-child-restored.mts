import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client: primary };
const other = { client: primary };
box.wrapper = retained;
box.wrapper.client = replica;
box.wrapper = other;
box.wrapper = retained;
// Restoring an object retains mutations to that object's descendants.
box.wrapper.client.get = replacement;

function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
