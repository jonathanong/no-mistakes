import { primary as client } from "@vendor/valkey";
declare function opaque(value: unknown): PromiseLike<unknown>;
function replacement() {}
const holder = { box: { client } };
const pending = opaque(holder);
// The first capture is created while walking the suspension operand.
await ((holder.box.client = {}), pending);
holder.box.client.get = replacement;
function scan(value) {
  value.get("key");
}
scan(client);
