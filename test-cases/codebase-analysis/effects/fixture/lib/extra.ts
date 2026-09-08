import { ValkeyCache } from "valkey";
import { client } from "./client";
import { invalidate } from "./b";

// Arrow function bound to a const: the caller of the nested effect is `handler`.
export const handler = () => {
  new ValkeyCache();
};

export function run() {
  // Member call: matches by the property name `createSubscriber`.
  client.createSubscriber();
  // Parenthesized callee resolving to the flat `functions` entry `standalone`.
  (standalone)();
  // Destructuring binding of an arrow (non-identifier binding pattern).
  const [first] = () => 0;
  // Computed-member callee (neither identifier nor static member).
  (client as never)[first]();
}

// Effects remain spelling-based even when this local callable shadows an
// import. Graph/symbol consumers must instead respect target identity.
export function shadowedEffect() {
  const invalidate = () => undefined;
  invalidate();
}

function standalone() {}
