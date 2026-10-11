// The default body is deferred, but the first later case test still runs.
switch (globalThis.mode) {
  default: break;
  case require('../helpers/mock.cjs'): break;
}
import('../src/target.cjs');
