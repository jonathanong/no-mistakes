import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { wrapper: { client: primary } };
box = { wrapper: { client: replica } };
const previous = box.wrapper;
// A fresh parent after a root rebind still predates the first RHS alias fact.
box.wrapper = {};
box.wrapper.client = primary;
previous.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
