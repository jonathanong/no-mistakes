import { primary as client } from "@vendor/valkey";
let box = { client };
// These cases must keep the canonical complete object conversion.
box = { get client() { return client; } };
box.client.get = replacement;
box = { ...{ client } };
box = { ["client"]: client };
box = { __proto__: client };
box = { __proto__: null, client };
box = { client, client };
box = { client: undefined };
box = { 1: client };
box = { 1n: client };
box = { client: makeClient() };
box = ({ client } as object);
function shadowed(undefined: unknown) { box = { client: undefined }; }
