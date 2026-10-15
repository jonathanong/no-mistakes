import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
// Parser recovery accepts this invalid duplicate; keep its tracked root conservative.
let client = { nested: replica };
client.nested.get = replacement;
