import { primary, replica } from '@vendor/valkey';

function replacement() {}
let box = { client: primary };

function write() {
  box.client.get = replacement;
}

write();
box = { client: replica };
replica.get('write happened before this receiver was acquired');
