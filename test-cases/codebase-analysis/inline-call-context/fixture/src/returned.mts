import { primary, session } from '@vendor/valkey';
function replacement() {}
// Preserve call order: future writes are safe only in the same enclosing invocation.
export function outer() {
  const opts = { client: primary };
  function factory() { return ({ client }) => client.get(); }
  factory()(opts);
  primary.get = replacement;
}
outer();
