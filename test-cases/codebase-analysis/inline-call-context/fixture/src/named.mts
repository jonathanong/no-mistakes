import { primary } from '@vendor/valkey';
function replacement() {}
export function outer() {
  const opts = { client: primary };
  function helper({ client }) { client.get(); }
  helper(opts);
  primary.get = replacement;
}
outer();
