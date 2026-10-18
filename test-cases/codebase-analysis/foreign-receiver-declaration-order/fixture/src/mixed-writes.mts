import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write(key) {
  box.client.get = replacement;
  box.client[key] = replacement;
  delete box.client.get;
}
function rebind() { box = { client: replica }; }
rebind();
write('get');
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
