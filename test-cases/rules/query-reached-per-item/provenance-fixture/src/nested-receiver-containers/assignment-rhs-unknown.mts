import { primary, replica } from '@vendor/valkey';
function replacement() {}
function unknown() { return globalThis.external; }
const box = { wrapper: { client: replica } };
const retained = { client: primary };
box.wrapper = retained;
box.wrapper = unknown();
// An opaque overwrite is a conservative barrier, not an exact old RHS identity.
box.wrapper.client.get = replacement;
function consume(value, other) { value.get('possible'); other.get('possible'); }
consume(primary, replica);
