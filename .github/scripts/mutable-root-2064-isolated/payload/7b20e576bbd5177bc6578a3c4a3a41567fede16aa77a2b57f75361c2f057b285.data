import { primary as client } from "@vendor/valkey";
declare function opaque(value: unknown): PromiseLike<unknown>;
function replacement() {}
const holder = { box: { client } };
const pending = opaque(holder);
holder.box.client = {};
// Thenable completion may restore holder.box after the capture.
await pending;
holder.box.client.get = replacement;
function scan(value) {
  value.get("key");
}
scan(client);
