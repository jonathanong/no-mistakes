import { primary, replica, unrelated } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
const before = replica.get;
const sink = {};
function write() { box.client.get = replacement; }
// The RHS rebind executes before the computed LHS writer.
[sink[(write(), 'key')]] = [(box = { client: replica })];
replica.get('changed');
before('snapshot-before-pattern');
