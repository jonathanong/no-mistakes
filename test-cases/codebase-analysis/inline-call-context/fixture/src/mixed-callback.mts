import { primary } from '@vendor/valkey';
function factory() { return ({ client }) => client.get(); }
const helper = factory();
const opts = { client: primary };
helper(opts);
// An opaque callback invocation still contributes unknown arguments.
opaque(helper);
