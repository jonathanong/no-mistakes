import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { holder: { wrapper: { client: replica } } };
const previous = box.holder.wrapper;
// This fresh identity predates the first RHS alias fact for the new wrapper.
box.holder.wrapper = {};
box.holder.wrapper.client = primary;
previous.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
