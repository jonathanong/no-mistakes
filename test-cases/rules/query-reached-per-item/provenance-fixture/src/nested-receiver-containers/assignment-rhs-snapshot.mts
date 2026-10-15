import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client: primary };
const other = { client: replica };
box.wrapper = retained;
const saved = box.wrapper;
box.wrapper = other;
// A saved container keeps its earlier wrapper acquisition while its descendant stays live.
saved.client.get = replacement;
function consume(value, untouched) { value.get('key'); untouched.get('sibling'); }
consume(primary, replica);
