import { primary as client } from "@vendor/valkey";
declare function opaque(value: unknown): PromiseLike<unknown>;
function replacement() {}
const holder = { box: { client } };
const pending = opaque(holder);
holder.box.client = {};
const saved = client.get;
await pending;
holder.box.client.get = replacement;
function scan(fn) {
  fn("key");
}
scan(saved);
