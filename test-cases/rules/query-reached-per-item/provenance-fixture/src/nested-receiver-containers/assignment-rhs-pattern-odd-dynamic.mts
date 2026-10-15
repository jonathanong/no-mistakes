import { primary, replica } from '@vendor/valkey';
function replacement() {}
declare const key: string;
let box = { client: primary };
({ [key]: box } = {
  one: { client: primary }, two: { client: replica }, three: { client: primary },
});
box.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
