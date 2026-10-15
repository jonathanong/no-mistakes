import { primary, replica } from '@vendor/valkey';
function replacement() {}
const box = { wrapper: { client: primary } };
const other = { client: replica };
if (globalThis.flag) box.wrapper = other;
box.wrapper.client.get = replacement;
function consume(value, other) { value.get('possible'); other.get('possible'); }
consume(primary, replica);
