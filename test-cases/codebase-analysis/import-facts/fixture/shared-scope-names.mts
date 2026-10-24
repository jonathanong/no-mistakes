import { execute } from './shared-scope-executor.mjs';

// Nested and class displays repeat across the finalized callable catalogs.
export function outer() {
  function nested() { execute(); }
  nested();
}

export class Worker {
  run() {
    function nested() { execute(); }
    nested();
  }
}

outer();
new Worker().run();
