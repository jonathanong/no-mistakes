import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// An unknown selector can acquire either supported literal member.
({ [globalThis.key]: box } = { first: { client: primary }, second: { client: replica } });
box.client.get = replacement;
function consume(value, other) { value.get('original'); other.get('changed'); }
consume(primary, replica);
