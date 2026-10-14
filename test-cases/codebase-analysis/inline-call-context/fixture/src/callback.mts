import { primary, session } from '@vendor/valkey';
function replacement() {}
// Preserve call order: future writes are safe only in the same enclosing invocation.
export function outer() {
  const opts = { client: primary };
  opaque(({ client } = opts) => client.get());
  primary.get = replacement;
}
outer();
