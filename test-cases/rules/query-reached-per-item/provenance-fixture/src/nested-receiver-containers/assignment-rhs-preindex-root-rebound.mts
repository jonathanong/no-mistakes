import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { wrapper: { client: replica } };
const previous = box;
box = { wrapper: {} };
box.wrapper = {};
box.wrapper.client = primary;
// A later root's canonical fresh receipt cannot identify the saved old root.
previous.wrapper.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
