import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// Declaration order says nothing about which callable executes first.
function write() { box.client.get = replacement; }
function rebind() { box = { client: replica }; }
declare function opaque(callback: () => void): void;
rebind();
opaque(write);
function consume(value, changed) {
  value.get('primary');
  changed.get('replica');
}
consume(primary, replica);
