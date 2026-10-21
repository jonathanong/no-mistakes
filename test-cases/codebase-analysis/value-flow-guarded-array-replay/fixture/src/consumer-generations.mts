import { primary, replica, untouched, unrelated } from '@vendor/valkey';
declare const flag: boolean;
declare function opaque(...values: unknown[]): void;
function replacement() {}
const box = { client: replica };
const other = { client: untouched };
function first(value) { value.client.get = replacement; }
function second(value) { value.client.info = replacement; }
const callGet = first;
const callInfo = second;
let args = [box];
if (flag) { args = [other]; }
callGet(...args);
// A distinct final consumer needs the entire live prefix, not the first one's delta.
callInfo(...args);
opaque(...args);
hoisted(...args);
function hoisted(value) { value.client.del = replacement; }
function third(value) { value.client.set = replacement; }
// A later unconditional generation must not send the earlier replica to third.
args = [{ client: primary }];
if (flag) { args = [other]; }
third(...args);
replica.get('first consumer');
replica.info('distinct consumer');
replica.del('hoisted consumer');
primary.set('new generation');
replica.set('old generation excluded');
untouched.get('historical latest origin');
unrelated.get('control');
