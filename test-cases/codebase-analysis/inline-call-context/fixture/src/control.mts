import { primary } from '@vendor/valkey';
export function outer() {
  const opts = { client: primary };
  (({ client }) => client.get())(opts);
}
outer();
