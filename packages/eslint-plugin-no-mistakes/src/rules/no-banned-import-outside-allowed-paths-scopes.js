"use strict";

// Tracks binding-tag scopes with discard-on-exit semantics for conditionally
// executed constructs, mirroring the reference rule
// no-global-fetch-outside-helper.js's push/pop-scope pattern (if/for/while
// bodies, switch cases with fallthrough tracking, try block/handler,
// non-static field initializers, and conditional/logical branch
// expressions). A tagged alias assigned inside one of these constructs must
// not leak into the surrounding scope, since the construct may execute zero
// or more times, or not on every path.
function createAliasScopeTracker() {
  let aliases = new Map();
  let clearedForwardAliases = new Set();
  const aliasStack = [];
  const clearedAliasStack = [];
  const switchStack = [];

  function push() {
    aliasStack.push(aliases);
    clearedAliasStack.push(clearedForwardAliases);
    aliases = new Map(aliases);
    clearedForwardAliases = new Set(clearedForwardAliases);
  }

  function pop() {
    aliases = aliasStack.pop();
    clearedForwardAliases = clearedAliasStack.pop();
  }

  // Discards the current branch's mutations and starts a fresh clone from
  // the same base a sibling branch (e.g. a ternary's alternate) should see.
  // Used between mutually exclusive branches that share one push(), instead
  // of a second pop-then-push pair keyed to the branch node itself: keying
  // to the branch node races an enter/exit listener on that same node (see
  // the call sites in the main rule file for why).
  function resetBranch() {
    pop();
    push();
  }

  // A switch case's aliases start from the switch's base state, or, on
  // fallthrough from a case with no terminating break/return/throw, from
  // whatever the previous case left behind; they're discarded again on exit
  // unless that case also falls through.
  function enterSwitch() {
    switchStack.push({ baseAliases: null, baseCleared: null, fallthrough: false });
  }

  function exitSwitch() {
    const state = switchStack.pop();
    if (!state.baseAliases) return;
    aliases = state.baseAliases;
    clearedForwardAliases = state.baseCleared;
  }

  function enterSwitchCase() {
    const state = switchStack.at(-1);
    if (!state) return;
    if (!state.baseAliases) {
      state.baseAliases = aliases;
      state.baseCleared = clearedForwardAliases;
    }
    if (state.fallthrough) return;
    aliases = new Map(state.baseAliases);
    clearedForwardAliases = new Set(state.baseCleared);
  }

  function exitsSwitchCase(node) {
    return (
      node.consequent?.some(
        (child) =>
          child.type === "BreakStatement" ||
          child.type === "ReturnStatement" ||
          child.type === "ThrowStatement",
      ) ?? false
    );
  }

  function exitSwitchCase(node) {
    const state = switchStack.at(-1);
    if (!state) return;
    state.fallthrough = !exitsSwitchCase(node);
    if (!state.fallthrough) {
      aliases = new Map(state.baseAliases);
      clearedForwardAliases = new Set(state.baseCleared);
    }
  }

  return {
    push,
    pop,
    resetBranch,
    enterSwitch,
    exitSwitch,
    enterSwitchCase,
    exitSwitchCase,
    get aliases() {
      return aliases;
    },
    get clearedForwardAliases() {
      return clearedForwardAliases;
    },
  };
}

module.exports = { createAliasScopeTracker };
