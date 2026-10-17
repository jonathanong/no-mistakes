import { primary as client } from "@vendor/valkey";
declare function opaque(value: unknown): PromiseLike<unknown>;
function replacement() {}
await opaque({});
const holder = { box: { client } };
holder.box.client = {};
holder.box.client.get = replacement;
function scan(value) {
  value.get("key");
}
scan(client);
