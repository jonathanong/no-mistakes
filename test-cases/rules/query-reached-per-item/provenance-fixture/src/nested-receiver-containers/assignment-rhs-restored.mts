import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: {} };
const retained = { client: primary };
const other = { client: replica };
box.wrapper = retained;
box.wrapper = other;
box.wrapper = retained;
// Restoration must select the latest alias rather than permanently unioning both containers.
box.wrapper.client.get = replacement;
function consume(value, untouched) { value.get('key'); untouched.get('sibling'); }
consume(primary, replica);
