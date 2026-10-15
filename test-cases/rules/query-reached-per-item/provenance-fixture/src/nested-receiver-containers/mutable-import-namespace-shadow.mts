import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
// Namespace-local names retain their actual lexical binding identity.
namespace Scope { let client = { nested: replica }; client.nested.get = replacement; }
