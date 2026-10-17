import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client: primary };
const other = { client: primary };
box.wrapper = retained;
const saved = box.wrapper;
box.wrapper = other;
box.wrapper.client = replica;
// A descendant replacement on the new wrapper cannot change the saved wrapper.
saved.client.get = replacement;
function consume(value, untouched) { value.get('saved'); untouched.get('other'); }
consume(primary, replica);
