import { ValkeyCache } from "valkey";
import { client } from "./client";
import { invalidate as mark } from "./effect-alias-source";

// Arrow function bound to a const: the caller of the nested effect is `handler`.
export const handler = () => {
  new ValkeyCache();
};

export function run() {
  // Member call: matches by the property name `createSubscriber`.
  client.createSubscriber();
  // Parenthesized callee resolving to the flat `functions` entry `standalone`.
  (standalone)();
  mark();
  // Destructuring binding of an arrow (non-identifier binding pattern).
  const [first] = () => 0;
  // Computed-member callee (neither identifier nor static member).
  (client as never)[first]();
}

function standalone() {}

// Effects intentionally retain their historical spelling-based contract even
// when the configured name is supplied through a local binding.
export function shadowedEffect(invalidate: () => void) {
  invalidate();
}
