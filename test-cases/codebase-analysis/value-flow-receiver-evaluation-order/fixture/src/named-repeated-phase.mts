import { namedLoop } from '@vendor/valkey';
function replacement() {}
function acquire(fn = namedLoop.get) { return fn; }
function forward() { return acquire(); }
function consume(fn) { fn('key'); }
for (const value of [0, 1]) {
  consume(forward());
  namedLoop.get = replacement;
}
