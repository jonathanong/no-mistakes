import { primary } from '@vendor/valkey';
function replacement() {}
function writer() { primary.get = replacement; }
export function outer() {
  const opts = { client: primary };
  (({ client }) => client.get())(opts);
}
outer();
writer();
