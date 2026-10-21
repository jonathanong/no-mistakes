import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
declare const unknown: unknown[];
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
const late = { client: primary };
function get(value) { value.client.get = replacement; }
function info(value) { value.client.info = replacement; }
function del(value) { value.client.del = replacement; }
let foreign = [];
// Merely declaring a deferred writer does not guarantee it runs.
function deferred() { foreign.push(box); }
if (flag) { foreign = [box]; }
get(...foreign);
let spread = [];
spread.push(...unknown);
if (flag) { spread = [other]; }
info(...spread);
let repeated = [];
while (flag) { repeated.push(late); }
if (flag) { repeated = [late]; }
del(...repeated);
replica.get('deferred push is not guaranteed');
untouched.info('unknown spread does not establish a slot');
primary.del('loop push is not guaranteed');
unrelated.get('control');
