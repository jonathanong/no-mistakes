import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
function write() { box.client.get = replacement; }
// A multi-hop closed caller chain must retain its actual module invocation.
function first() { write(); }
function second() { first(); }
function third() { second(); }
box = { client: replica };
third();
primary.get('untouched');
replica.get('changed');
