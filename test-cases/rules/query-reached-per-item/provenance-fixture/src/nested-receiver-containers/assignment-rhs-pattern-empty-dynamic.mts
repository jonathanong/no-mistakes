import { primary, replica } from '@vendor/valkey';
function replacement() {}
declare const key: string;
let box = { client: primary };
({ [key]: box = { client: replica } } = {});
box.client.get = replacement;
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
