import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
declare const flag: boolean;
let box = { client: primary };
const other = { client: primary };
function mutate(value) { value.client.get = replacement; }
box = { client: replica };
const args = [flag ? other : box];
mutate(...args);
replica.get('changed');
unrelated.get('control');
