import { primary, replica } from '@vendor/valkey';
function replacement() {}
let box = { client: primary };
// Every callable has one incoming caller, but the closed cycle has no module
// invocation. Without a phase proof, the provider stays conservative.
function write() { box.client.get = replacement; third(); }
function second() { write(); }
function third() { second(); }
box = { client: replica };
primary.get('possible');
replica.get('possible');
