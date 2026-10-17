import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
// An uncalled function catalog must not mask the enclosing root reset.
[box, sink[(function deferred() { [box] = [{ client: replica }]; }, box.client.get = replacement, 'key')]] = [{ client: replica }, 0];
function consume(value) { value.get('key'); }
consume(client);
