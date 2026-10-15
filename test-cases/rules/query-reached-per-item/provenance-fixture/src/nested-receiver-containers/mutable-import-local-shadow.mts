import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
// The import shortcut must not apply to a nested lexical identity.
{ let client = { nested: replica }; client.nested.get = replacement; }
