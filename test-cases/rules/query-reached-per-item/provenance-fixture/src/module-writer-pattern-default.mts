import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
box = { client: replica };
write();
// Destructuring this mutable default currently loses receiver precision for both
// methods. The supplied-default fixture separately protects known siblings.
function consume({ client } = box) {
  client.get('changed');
  client.info('untouched sibling');
}
consume();
