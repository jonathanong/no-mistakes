import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { wrapper: {} };
const retained = { client: primary };
box.wrapper = retained;
const previous = box;
box = { wrapper: { client: replica } };
// The saved mutable root keeps its earlier receiver-prefix timeline.
previous.wrapper.client.get = replacement;

function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
