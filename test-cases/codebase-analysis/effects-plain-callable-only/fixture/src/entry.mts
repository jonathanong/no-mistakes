import { get } from '@vendor/plain-callable-only';
let callback;
// The callable proof capsule must not manufacture receiver regions or owners.
for (let index = 0; index < 1; index++) {
  callback = function preserved() { return callback; };
}
export function consume() { get(); }
consume();
