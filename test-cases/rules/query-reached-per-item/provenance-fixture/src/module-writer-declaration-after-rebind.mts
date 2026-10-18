import { primary, replica } from '@vendor/valkey';

let box = { client: primary };
box = { client: replica };

const replacement = () => undefined;
function write() {
  box.client.get = replacement;
}

write();
replica.get();
