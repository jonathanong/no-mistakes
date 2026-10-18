import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
declare function opaque(callback: () => void): void;
function write() { box.client.get = replacement; }
box = { client: replica };
opaque(write);
primary.get('possible');
replica.get('possible');
