import { primary, replica } from '@vendor/valkey';
declare function unknown(): unknown;
function replacement() {}
const box = { wrapper: { client: replica } };
const previous = box.wrapper;
// An unknown result may return the saved wrapper; preserve that possibility.
box.wrapper = unknown();
box.wrapper.client = primary;
previous.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
