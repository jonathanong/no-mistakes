import { session } from '@vendor/valkey';
function replacement() {}
// Retain this module's receiver histories without mutating the supplied clients.
session.info = replacement;
export function object(client) { return { client }; }
export function array(client) { return [client]; }
export function copied(client) { return { get: client.get }; }
