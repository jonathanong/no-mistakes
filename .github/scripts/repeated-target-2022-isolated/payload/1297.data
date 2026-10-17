import { primary as client } from "@vendor/valkey";
declare function opaque(value: unknown): PromiseLike<unknown>;
function replacement() {}
const holder = { box: { client } };
const pending = opaque(holder);
holder.box.client = {};
// A never-invoked callable must not invalidate the module capture.
async function deferred() {
  await pending;
}
holder.box.client.get = replacement;
function scan(value) {
  value.get("key");
}
scan(client);
