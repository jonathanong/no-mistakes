import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const before = replica.get;
const sink = {};
function consume(method) { method('changed-method'); }
function write() { box.client.get = replacement; }
// The RHS writer executes before the physically earlier computed LHS copy.
[sink[(consume(replica.get), 'key')]] = [(box = { client: replica }, write())];
before('snapshot-before-pattern');
