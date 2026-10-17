import { primary, replica } from '@vendor/valkey';

function replacement() {}
let box = { client: primary };
const primaryMethod = primary.get;

function write() {
  box.client.get = replacement;
}

box = { client: replica };
write();
primary.get('primary remains unchanged');
replica.get('must be unknown after the deferred write');
primaryMethod('copied method remains unchanged');
