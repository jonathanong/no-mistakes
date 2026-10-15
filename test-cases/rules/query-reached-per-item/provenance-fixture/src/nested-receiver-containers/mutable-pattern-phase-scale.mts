import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
// Thirty-two nested RHS patterns must share one root phase traversal.
([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [([box] = [(box.client.get = replacement, { client: replica })])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])]);
