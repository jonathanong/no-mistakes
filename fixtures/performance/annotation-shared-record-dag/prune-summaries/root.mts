import { read } from '@bench/db';
// Nested alternatives intentionally retain a captured arguments cycle and force
// invalidation fallback; stable records must survive pruning without rescanning.
export function factory(input) {
  const args = arguments;
  args[1] = args;
  return function read() { return args[0]; };
}
export function probe() {
const stable = { long_saved_record_label_for_stable_alternative_pruning: '/* stable */ SELECT 1' };
const clean = flag ? (other ? factory(stable.long_saved_record_label_for_stable_alternative_pruning) : factory('/* right */ SELECT 2')) : factory('/* outer */ SELECT 3');
const dirty = flag ? (other ? factory('SELECT 4') : factory('SELECT 5')) : factory('SELECT 6');
read(clean());
read("/* clean control */ SELECT 7"); // safe-query
read(dirty());
read("SELECT 8"); // unsafe-query
}
