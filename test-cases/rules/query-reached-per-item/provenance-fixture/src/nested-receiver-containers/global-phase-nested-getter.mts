import { primary as client } from '@vendor/valkey';
function replacement() {}
const holder = { box: { client } };
let unused;
holder.box.client = {};
const evil = makeEvil();
({ first: { x: unused }, second: holder.box.client.get } = { first: evil, second: replacement });
function makeEvil() { return { get x() { holder.box = { client }; return 0; } }; }
function restore() { holder.box = { client }; }
function globalPhaseNestedGetter(value) { value.get('key'); }
globalPhaseNestedGetter(client);
