import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: { client: replica } };
const retained = { client: replica };
box.wrapper = retained;
const previous = box.wrapper;
// A post-index fresh replacement publishes a newer exact ancestor barrier.
box.wrapper = {};
box.wrapper.client = primary;
previous.client.get = replacement;
function consume(value, other) { value.get('untouched'); other.get('changed'); }
consume(primary, replica);
