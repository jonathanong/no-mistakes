// An uncalled function must not make the skipped root require eligible.
function unused() {
  require('../helpers/skipped.cjs');
}
false && require('../helpers/skipped.cjs');
true || require('../helpers/skipped.cjs');
1 ?? require('../helpers/skipped.cjs');
if (false) require('../helpers/skipped.cjs');
false ? require('../helpers/skipped.cjs') : undefined;
while (false) require('../helpers/skipped.cjs');
for (; false;) require('../helpers/skipped.cjs');
// Empty iterations never evaluate binding defaults or assignment targets.
for (const [unused = require('../helpers/skipped.cjs')] of []) {}
for (globalThis[require('../helpers/skipped.cjs')] in {}) {}
// A do-while body runs once, but its test is skipped after `break`.
do { break; } while (require('../helpers/skipped.cjs'));
globalThis.maybe?.(require('../helpers/skipped.cjs'));
import('../src/target.cjs');
