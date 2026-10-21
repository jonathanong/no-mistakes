import { replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
function Construct(value) { value.client.get = replacement; }
function call(value) { value.client.info = replacement; }
let args = [box];
if (flag) { args = [other]; }
// Constructor records are not ordinary escaped spread descriptors.
new Construct(...args);
call(...args);
replica.get('constructor support stays unchanged');
replica.info('ordinary call recovers earlier origin');
untouched.info('historical latest origin');
unrelated.get('control');
