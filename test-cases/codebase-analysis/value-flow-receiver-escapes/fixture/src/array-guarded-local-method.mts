import { unrelated } from '@vendor/valkey';
declare const flag: boolean;
function original() {}
function replacement() {}
const api = { get: original };
const other = { get: replacement };
function mutate(value) { value.get = replacement; }
let args = [api];
if (flag) { args = [other]; }
mutate(...args);
api.get();
unrelated.get('control');
