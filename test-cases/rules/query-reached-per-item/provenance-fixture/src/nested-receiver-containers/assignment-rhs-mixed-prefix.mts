import { primary, replica } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
const box = { wrapper: {} };
box.wrapper = { client: primary };
if (flag) box.wrapper.client = replica;
// The exact parent and possible child both remain runtime alternatives.
box.wrapper.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
