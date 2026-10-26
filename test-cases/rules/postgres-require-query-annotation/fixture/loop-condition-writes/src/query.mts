import { read } from '@example/db';

// Captured writes outside the loop still run between the two guards.
export function closureFlip(xs: readonly unknown[], initial: boolean) {
  let c = initial;
  const flip = () => { c = !c; };
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: a closure can make both resets false.
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    flip();
    if (!c) q = '/* second */ SELECT 4';
  }
}

export function declarationFlip(xs: readonly unknown[], c: boolean) {
  function flip() { c = !c; }
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: a declared helper mutates the captured parameter.
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    flip();
    if (!c) q = '/* second */ SELECT 4';
  }
}

let globalCondition = true;
function flipGlobal() { globalCondition = !globalCondition; }
export function moduleLetFlip(xs: readonly unknown[]) {
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: a sibling helper flips a module-level let.
    q = 'SELECT 2';
    if (globalCondition) q = '/* first */ SELECT 3';
    flipGlobal();
    if (!globalCondition) q = '/* second */ SELECT 4';
  }
}

var globalVar = true;
function flipGlobalVar() { globalVar = !globalVar; }
export function moduleVarFlip(xs: readonly unknown[]) {
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: var has the same mutable module ownership.
    q = 'SELECT 2';
    if (globalVar) q = '/* first */ SELECT 3';
    flipGlobalVar();
    if (!globalVar) q = '/* second */ SELECT 4';
  }
}

// Module state with any post-initializer write is not a fixed condition.
let moduleAssigned = true;
export function modulePreloopAssignment(xs: readonly unknown[], initial: boolean) {
  moduleAssigned = initial;
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: conservatively unlink writable module state.
    q = 'SELECT 2';
    if (moduleAssigned) q = '/* first */ SELECT 3';
    if (!moduleAssigned) q = '/* second */ SELECT 4';
  }
}

export function capturedUpdate(xs: readonly unknown[], initial: number) {
  let c = initial;
  const flip = () => { c++; };
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: update syntax can change the captured condition.
    q = 'SELECT 2';
    if (c === 0) q = '/* first */ SELECT 3';
    flip();
    if (!(c === 0)) q = '/* second */ SELECT 4';
  }
}

export function capturedDestructure(xs: readonly unknown[], initial: boolean) {
  let c = initial;
  const flip = () => { ({ c } = { c: !c }); };
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: destructuring writes also invalidate correlation.
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    flip();
    if (!c) q = '/* second */ SELECT 4';
  }
}

// Unsupported writes lack precise function context, even before the loop.
export function preloopUpdate(xs: readonly unknown[], initial: number) {
  let c = initial;
  c++;
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: conservatively unlink an unsupported write.
    q = 'SELECT 2';
    if (c === 0) q = '/* first */ SELECT 3';
    if (!(c === 0)) q = '/* second */ SELECT 4';
  }
}

export function preloopDestructure(xs: readonly unknown[], initial: boolean) {
  let c = initial;
  ({ c } = { c: !c });
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q); // missing: unsupported destructuring is also conservatively unlinked.
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    if (!c) q = '/* second */ SELECT 4';
  }
}

export function constControl(xs: readonly unknown[], initial: boolean) {
  const c = initial;
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q);
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    if (!c) q = '/* second */ SELECT 4';
  }
}

// Shadowing writes belong to another binding, even when the spelling agrees.
export function shadowControl(xs: readonly unknown[], initial: boolean) {
  let c = initial;
  function flip() { let c = initial; c = !c; }
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q);
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    flip();
    if (!c) q = '/* second */ SELECT 4';
  }
}

export function preloopAssignmentControl(xs: readonly unknown[], initial: boolean) {
  let c = initial;
  c = !c;
  let q = '/* initial */ SELECT 1';
  for (const x of xs) {
    read(q);
    q = 'SELECT 2';
    if (c) q = '/* first */ SELECT 3';
    if (!c) q = '/* second */ SELECT 4';
  }
}
