import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
box.wrapper = { client: primary };
// Component union after the store must move its ordered RHS facts too.
const late = box;
late.wrapper.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
