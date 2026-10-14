import { primary } from '@vendor/valkey';
function replacement() {}
export function outer() {
  const opts = { client: primary };
  (({ client }) => client.get())(opts);
  primary.get = replacement;
}
outer();
