import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
// Hoisted var duplicates are accepted by the parser and must retain root lookup.
var client = { nested: replica };
client.nested.get = replacement;
