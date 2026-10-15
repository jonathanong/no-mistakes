import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: { client: replica } };
const previous = box.wrapper;
// This fresh identity predates the first RHS alias fact for the new wrapper.
box.wrapper = {};
box.wrapper.client = primary;
previous.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
