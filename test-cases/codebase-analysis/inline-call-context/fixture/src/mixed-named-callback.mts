import { primary } from '@vendor/valkey';
const opts = { client: primary };
const helper = ({ client }) => client.get();
((fn) => fn(opts))(helper);
// Authored callback arguments remain unknown even beside a direct invocation.
opaque(helper);
