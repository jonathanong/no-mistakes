import { primary, replica } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { wrapper: { client: replica } };
const previous = box.wrapper;
// A conditional fresh object cannot detach the saved wrapper in every execution.
if (flag) box.wrapper = {};
box.wrapper.client = primary;
previous.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
