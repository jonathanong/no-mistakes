import { primary } from '@vendor/valkey';

function neverInitialized() {
  function acquire() { return neverSaved; }
  return function neverRun(fn = acquire()) { fn('key'); };
  const neverSaved = primary.get;
}
neverInitialized()();

function adjacentNeverInitialized() {
  function acquire() { return adjacentSaved; }
  // Keep the declaration adjacent to the terminal statement's half-open end.
  return function adjacentRun(fn = acquire()) { fn('key'); };const adjacentSaved = primary.get;
}
adjacentNeverInitialized()();

function terminalNestedBlock() {
  {
    function acquire() { return nestedSaved; }
    // The final block tests a terminal nested sequence with no outer suffix.
    return function nestedRun(fn = acquire()) { fn('key'); };const nestedSaved = primary.get;
  }
}
terminalNestedBlock()();

function initializedBeforeReturn() {
  const saved = primary.get;
  function acquire() { return saved; }
  return function initializedRun(fn = acquire()) { fn('key'); };
}
initializedBeforeReturn()();

function conditionalReturn(flag: boolean) {
  if (flag) return;
  const saved = primary.get;
  function acquire() { return saved; }
  function conditionalRun(fn = acquire()) { fn('key'); }
  conditionalRun();
}
conditionalReturn(false);

function branchLocalCutoff(flag: boolean) {
  if (flag) {
    return;
    const unreachable = primary.get;
  } else {
    const saved = primary.get;
    function acquire() { return saved; }
    function branchRun(fn = acquire()) { fn('key'); }
    branchRun();
  }
}
branchLocalCutoff(false);

function bothBranchesReturn(flag: boolean) {
  function acquire() { return saved; }
  if (flag) return;
  else return;
  const saved = primary.get;
  return function unreachableRun(fn = acquire()) { fn('key'); };
}
bothBranchesReturn(false)?.();

function hoistedCalleeAfterReturn() {
  return later();
  function later() {
    const saved = primary.get;
    function acquire() { return saved; }
    return function hoistedRun(fn = acquire()) { fn('key'); };
  }
}
hoistedCalleeAfterReturn()();

function hoistedStaticBlockAfterReturn() {
  return later();
  function later() {
    class Captured {
      static {
        const saved = primary.get;
        function acquire() { return saved; }
        function staticInvoke(fn = acquire()) { fn('key'); }
        staticInvoke();
      }
    }
  }
}
hoistedStaticBlockAfterReturn();
