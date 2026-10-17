import { primary as client } from '@vendor/valkey';
function replacement() {}
const alias = client;
// An unrelated mutable origin activates root coordinates but does not remove alias successors.
let unrelated = { retained: true };
client.get = replacement;
export function importedAlias() { alias.get('live'); }
