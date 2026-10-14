import { primary } from '@vendor/valkey';
function replacement() {}
function opaque() { return {}; }
let first, second, third, fourth;
// Keep an unrelated receiver history live: scalar patterns must still stay cold.
primary.get = replacement;
({ one: first, nested: { two: second, three: third = 3 } = {}, last: fourth } = {});
[first, [second, third = 3], fourth] = [1, []];
// An opaque target has no recordable receiver and must not allocate ranges.
({ one: opaque().get, two: first = 2 } = {});
