import { primary as client } from '@vendor/valkey';
let root = { client };
let other = root;
let tail = root;
const sink = {};
function key() { return 'key'; }
// Metadata ignores member paths; the normal visitor must retain computed/default effects.
([root = other, sink[key()], ...tail] = [other, other, other]);
({ root, named: other = root, [key()]: sink.key, ...tail } = sink);
([(root as unknown), (other satisfies unknown), tail!] = [root, other, tail]);
([(<unknown>root)] = [other]);
(root as unknown) = other;
// A member wrapped in TypeScript syntax remains a non-root target.
([(sink.key as unknown)] = [root]);
({ named: root } = sink);
