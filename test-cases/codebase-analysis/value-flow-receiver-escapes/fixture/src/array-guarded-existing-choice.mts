import { primary, unrelated } from '@vendor/valkey';
declare const flag: boolean;
declare const opaque: unknown;
function replacement() {}
const api = { get: opaque ?? primary.get };
const other = { get: replacement };
function mutate(value) { value.get = replacement; }
let args = [api];
if (flag) { args = [other]; }
mutate(...args);
api.get();
unrelated.get('control');
