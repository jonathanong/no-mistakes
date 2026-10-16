import { primary as client } from "@vendor/valkey";
import { primary as undefined } from "@vendor/valkey";
let box = { client };
box = { client };
box.client.get = replacement;
box = { "client": client };
box.client.get = replacement;
const __proto__ = client;
// Shorthand __proto__ is an ordinary own field, unlike the prototype setter.
box = { __proto__ };
box.__proto__.get = replacement;
box = { client: missing };
box.client.get = replacement;
function nested(client: unknown) {
  box = { client };
  box.client.get = replacement;
}
// An imported undefined spelling keeps the ordinary expression converter.
box = { client: undefined };
box.client.get = replacement;
