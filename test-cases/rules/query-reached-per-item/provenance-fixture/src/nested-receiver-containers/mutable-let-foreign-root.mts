import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
declare function opaque(callback: () => void): void;
let box = { client };
function change() { box = { client: replica }; }
opaque(change);
box.client.get = replacement;
function consume(value) { value.get('key'); }
consume(replica);
