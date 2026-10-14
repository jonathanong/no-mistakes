import { primary, session } from '@vendor/valkey';
function replacement() {}
// Preserve call order: future writes are safe only in the same enclosing invocation.
export function outer() {
  const opts = { client: primary };
  primary.get = replacement;
  (({ client }) => client.get())(opts);
}
outer();
