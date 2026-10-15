import { primary, replica } from '@vendor/valkey';
function replacement() {}
const retained = { client: primary };
const box = { wrapper: {} };
box.wrapper = retained;
// RHS facts must use the existing member-target phase receipt.
[box.wrapper.client.get] = [replacement];
function consume(value, other) { value.get('first'); other.get('second'); }
consume(primary, replica);
