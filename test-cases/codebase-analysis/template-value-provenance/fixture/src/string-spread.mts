import { replica, unrelated } from '@vendor/valkey';
function replacement() {}
const box = { client: replica };
function mutate(value) { value.client.get = replacement; }
const text = `prefix ${box}`;
// A string spread yields characters, never the interpolation's object.
mutate(...text);
replica.get('template does not act as array');
unrelated.get('control');
