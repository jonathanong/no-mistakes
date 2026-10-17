import { primary, replica } from '@vendor/valkey';
function replacement() {}
const retained = { client: primary };
retained.self = retained;
// The finite requested member path must still reach client through a cycle.
retained.self.self.client.get = replacement;
function consume(value, other) { value.get('changed'); other.get('untouched'); }
consume(primary, replica);
