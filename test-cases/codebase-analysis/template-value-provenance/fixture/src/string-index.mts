import { replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: replica };
function mutate(value) { value.client.get = replacement; }
const text = `prefix ${box}`;
// String indexing must not project the interpolation's receiver identity.
mutate(text[0]);
replica.get('template indexing does not return box');
unrelated.get('control');
