import { read } from '@example/db';
import { it } from 'vitest';
declare function invoke(callback: () => void): void;

// A known direct SQL literal does not need caller replay.
invoke(() => { read('SELECT 1 OFFSET 3'); }); // KNOWN_CALLBACK

// An unknown argument needs the whole callback, including its earlier local write.
invoke(() => { let statement = 'SELECT 1 OFFSET 4'; read(statement); }); // UNKNOWN_CALLBACK

// An unparameterized template with a hole may need complete caller replay.
invoke(() => { read(`SELECT 1 ${'OFFSET 9'}`); }); // PARTIAL_CALLBACK

// The const initializer gives this arrow a runtime binding.
const bound = () => { read('SELECT 1 OFFSET 5'); }; // BOUND_ARROW
invoke(bound);
const wrapped = (() => { read('SELECT 1 OFFSET 11'); }) as () => void;
invoke(wrapped);

function named() { read('SELECT 1 OFFSET 10'); }
invoke(named);

// Iterator rows use the callback parameter even when its body need not be replayed.
it.each(['row'])('case', (row) => { read('SELECT 1 OFFSET 8'); }); // ITERATION_CALLBACK
