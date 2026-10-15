import { primary, replica } from '@vendor/valkey';
function replacement() {}
const retained = { one: { client: primary }, two: { client: replica } };
const box = { first: {}, second: {} };
box.first = retained.one;
box.second = retained['two'];
box.first.client.get = replacement;
box.second.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
