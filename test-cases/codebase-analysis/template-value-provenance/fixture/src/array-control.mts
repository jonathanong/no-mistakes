import { replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: replica };
function mutate(value) { value.client.get = replacement; }
const values = [box];
mutate(...values);
replica.get('real array still projects box');
unrelated.get('control');
