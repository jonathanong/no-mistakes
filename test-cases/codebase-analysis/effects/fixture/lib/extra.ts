import { ValkeyCache } from "valkey";
import { client } from "./client";

// Arrow function bound to a const: the caller of the nested effect is `handler`.
export const handler = () => {
  new ValkeyCache();
};

export function run() {
  // Member call: matches by the property name `createSubscriber`.
  client.createSubscriber();
  // Parenthesized callee resolving to the flat `functions` entry `standalone`.
  (standalone)();
}

function standalone() {}
