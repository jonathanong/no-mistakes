import { primary as client } from "@vendor/valkey";
declare function opaque(value: unknown): PromiseLike<unknown>;
function replacement() {}
function* run() {
  const holder = { box: { client } };
  const pending = opaque(holder);
  holder.box.client = {};
  // Resuming the generator may run after external restoration of holder.box.
  yield pending;
  holder.box.client.get = replacement;
  function scan(value) {
    value.get("key");
  }
  scan(client);
}
run().next();
