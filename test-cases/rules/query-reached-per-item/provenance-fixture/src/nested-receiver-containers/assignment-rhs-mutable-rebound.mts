import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { wrapper: {} };
const retained = { client: primary };
box.wrapper = retained;
box = { wrapper: { client: replica } };
// An old root timeline must not replace the new root's wrapper identity.
box.wrapper.client.get = replacement;

function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
