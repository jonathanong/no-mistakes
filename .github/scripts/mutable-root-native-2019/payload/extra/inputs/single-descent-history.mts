import { primary as client } from '@vendor/valkey';
function replacement() {}
declare const key: string;
let box = { outer: { client } };
// RHS calls separate the write's start from its completion. Keep both times.
box.outer.client.get = replacement();
box.outer[key].get = replacement();
function writeAgain() {
  box.outer.client.get = replacement();
}
let { value = box.outer.client } = {};
value.get();
