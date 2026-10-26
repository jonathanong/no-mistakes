import { replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
function replacement() {}
const spare = { client: { get: replacement } };
const box = { client: replica };
const other = { client: untouched };
function mutate(_first, second = spare) { second.client.get = replacement; }
let args = [spare];
// Keep the authored suffix linear: every later known value survives in one possible tail.
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
if (flag) { args.push(box); }
args.push(other);
mutate(...args);
replica.get('optional first candidate');
untouched.get('late unique candidate');
unrelated.get('control');
