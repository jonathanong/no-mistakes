declare function read(): void;
declare const tx: { query(): void };
declare function batch(callback: () => void): void;
// Both branches reach shared: canonical adjacency, not source call order,
// chooses the first shortest predecessor. Non-sink prefixes need no paths.
export function start() { right(); left(); noise0(); }
function left() { shared(); }
function right() { shared(); }
function shared() { read(); tx.query(); }
function noise0() { noise1(); }
function noise1() { noise2(); }
function noise2() { noise3(); }
function noise3() { noise4(); }
function noise4() { noise5(); }
function noise5() { noise6(); }
function noise6() { noise7(); }
function noise7() { noise8(); }
function noise8() { noise9(); }
function noise9() { noise0(); tx.query(); }
export function transactionOnly() { tx.query(); }
export function selfSink() { read(); left(); }
export function alternate() { batch(left); right(); }
export function excludedOnly() { batch(left); }
for (const item of [1, 2]) {
  start();
  transactionOnly();
  selfSink();
  alternate();
  excludedOnly();
}

export const startAlias = start;
